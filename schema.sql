CREATE EXTENSION citext;
CREATE EXTENSION pg_cron;

CREATE DOMAIN USERNAME AS TEXT CONSTRAINT valid_username CHECK (
    VALUE ~ '^[[:word:]-]{3,20}$'
);

-- NOTE: This is the HTML5 specification, specifically incompatible with RFC5322.
CREATE DOMAIN EMAIL AS citext CONSTRAINT valid_email CHECK (
    value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$'
);

CREATE DOMAIN NONFUTURE_TIMESTAMP AS TIMESTAMP CHECK (VALUE <= CURRENT_TIMESTAMP);

CREATE DOMAIN TWOPOINT_UDEC AS DECIMAL(10, 2) CHECK (VALUE >= 0);

CREATE DOMAIN UINT AS INT CHECK (VALUE >= 0);

CREATE DOMAIN POSITIVE_INT AS INT CHECK (VALUE > 0);

CREATE DOMAIN RATING AS INT CHECK (VALUE BETWEEN 1 AND 5);

-- TODO: Improve URL representation or replace entirely (server storage).
CREATE DOMAIN URL AS TEXT;

CREATE TYPE AMOUNT AS (
    value DECIMAL(10, 2),
    unit TEXT
);

CREATE TYPE VOTE AS ENUM ('like', 'dislike');

CREATE TYPE ROLE AS ENUM ('customer', 'vendor', 'administrator');

CREATE TABLE users (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username USERNAME UNIQUE NOT NULL,
    email EMAIL UNIQUE NOT NULL,
    -- Password and other contact information omitted: login information is a security detail,
    -- contact information is trivial and uninteresting.

    -- NOTE: Currently, no data is deleted from other tables when a user is marked as deleted.
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    role ROLE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- This column can't be automatically generated as it has to be written to by triggers when
    -- "subclass" is updated.
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE FUNCTION creation_time() RETURNS TRIGGER AS $$
BEGIN
    NEW.created_at = CASE TG_OP
        WHEN 'INSERT' THEN CURRENT_TIMESTAMP
        WHEN 'UPDATE' THEN OLD.created_at
    END;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE TRIGGER users_creation_time
BEFORE INSERT OR UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE FUNCTION update_time() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE TRIGGER users_update_time
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE TABLE customers (
    id INT NOT NULL PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    profile_picture URL,
    -- Null: not a member.
    member_since NONFUTURE_TIMESTAMP,
    can_review BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE vendors (
    id INT NOT NULL PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    profile_picture URL,
    display_name TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE FUNCTION update_time_user_super() RETURNS TRIGGER AS $$
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE TRIGGER customers_update_time_super
BEFORE UPDATE ON customers
FOR EACH ROW EXECUTE FUNCTION update_time_user_super();

CREATE TRIGGER vendors_update_time_super
BEFORE UPDATE ON vendors
FOR EACH ROW EXECUTE FUNCTION update_time_user_super();

CREATE FUNCTION validate_user_subclass() RETURNS TRIGGER AS $$
BEGIN
    CASE NEW.role
        WHEN 'customer' THEN
            IF NOT EXISTS (SELECT 1 FROM customers WHERE id = NEW.id) THEN
                RAISE EXCEPTION 'Customer must have row in customer table.';
            END IF;
            IF EXISTS (SELECT 1 FROM vendors WHERE id = NEW.id) THEN
                RAISE EXCEPTION 'Customer must not have row in vendor table.';
            END IF;
        WHEN 'vendor' THEN
            IF NOT EXISTS (SELECT 1 FROM vendors WHERE id = NEW.id) THEN
                RAISE EXCEPTION 'Vendor must have row in vendor table.';
            END IF;
            IF EXISTS (SELECT 1 FROM customers WHERE id = NEW.id) THEN
                RAISE EXCEPTION 'Vendor must not have row in customer table.';
            END IF;
        WHEN 'administrator' THEN
            IF EXISTS (SELECT 1 FROM customers WHERE id = NEW.id) THEN
                RAISE EXCEPTION 'Administrator must not have row in customer table.';
            END IF;
            IF EXISTS (SELECT 1 FROM vendors WHERE id = NEW.id) THEN
                RAISE EXCEPTION 'Administrator must not have row in vendor table.';
            END IF;
        ELSE
            RAISE WARNING 'Unknown role: %.', NEW.role;
    END CASE;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE CONSTRAINT TRIGGER users_valid_subclass
AFTER INSERT OR UPDATE OF role ON users
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_user_subclass();

CREATE FUNCTION validate_user_superclass() RETURNS TRIGGER AS $$
BEGIN
    CASE TG_TABLE_NAME
        WHEN 'customers'
        AND NOT EXISTS (SELECT 1 FROM users WHERE id = NEW.id AND role = 'customer') THEN
            RAISE EXCEPTION 'User is not a customer.';
        WHEN 'vendors'
        AND NOT EXISTS (SELECT 1 FROM users WHERE id = NEW.id AND role = 'vendor') THEN
            RAISE EXCEPTION 'User is not a vendor.';
    END CASE;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER customers_valid_superclass
BEFORE INSERT OR UPDATE OF id ON customers
FOR EACH ROW EXECUTE FUNCTION validate_user_superclass();

CREATE TRIGGER vendors_valid_superclass
BEFORE INSERT OR UPDATE OF id ON vendors
FOR EACH ROW EXECUTE FUNCTION validate_user_superclass();

CREATE FUNCTION validate_user_role_change() RETURNS TRIGGER AS $$
BEGIN
    -- `id` is generated and so can't be changed.
    CASE OLD.role
        WHEN 'customer' THEN
            IF EXISTS (SELECT 1 FROM customers WHERE id = OLD.id) THEN
                RAISE EXCEPTION 'Must remove customer data to change customer role.';
            END IF;
        WHEN 'vendor' THEN
            IF EXISTS (SELECT 1 FROM vendors WHERE id = OLD.id) THEN
                RAISE EXCEPTION 'Must remove vendor data to change vendor role.';
            END IF;
        WHEN 'administrator' THEN
            NULL;
        ELSE
            RAISE WARNING 'Unknown role: %.', NEW.role;
    END CASE;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER users_valid_role_change
BEFORE UPDATE OF role ON users
FOR EACH ROW EXECUTE FUNCTION validate_user_role_change();

CREATE FUNCTION validate_user_subclass_deletion() RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM users WHERE id = OLD.id) THEN
        RAISE EXCEPTION 'Must remove user superclass along with subclass.';
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER customers_deletion
BEFORE DELETE ON customers
FOR EACH ROW EXECUTE FUNCTION validate_user_subclass_deletion();

CREATE TRIGGER vendors_deletion
BEFORE DELETE ON vendors
FOR EACH ROW EXECUTE FUNCTION validate_user_subclass_deletion();

CREATE FUNCTION validate_user_one_subclass() RETURNS TRIGGER AS $$
BEGIN
    CASE TG_TABLE_NAME
        WHEN 'customers' AND EXISTS (SELECT 1 FROM vendors WHERE id = NEW.id) THEN
            RAISE EXCEPTION 'User is already a vendor.';
        WHEN 'vendors' AND EXISTS (SELECT 1 FROM customers WHERE id = NEW.id) THEN
            RAISE EXCEPTION 'User is already a customer.';
        ELSE
            RAISE WARNING 'Unknown role: %.', NEW.role;
    END CASE;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER customers_unique_superclass
BEFORE INSERT OR UPDATE OF id ON customers
FOR EACH ROW EXECUTE FUNCTION validate_user_one_subclass();

CREATE TRIGGER vendors_unique_superclass
BEFORE INSERT OR UPDATE OF id ON vendors
FOR EACH ROW EXECUTE FUNCTION validate_user_one_subclass();

CREATE TABLE categories (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    parent INT REFERENCES categories(id) ON DELETE CASCADE
);

CREATE TYPE category_path_segment AS (id INT NOT NULL, name TEXT NOT NULL);
CREATE FUNCTION category_path(start_id INT) RETURNS category_path_segment[] AS $$
DECLARE
    path category_path_segment[] := ARRAY[]::category_path_segment[];
    current categories%ROWTYPE;
BEGIN
    current.id := start_id;
    
    LOOP
        SELECT * INTO current FROM categories WHERE id = current.id;
        IF EXISTS (SELECT 1 FROM unnest(path) WHERE id = current.id) THEN
            RAISE EXCEPTION 'Cycle detected.';
        END IF;

        path := ARRAY[(current.id, current.name)] || path;

        IF current.parent IS NULL THEN
            EXIT;
        END IF;

        current.id := current.parent;
    END LOOP;

    RETURN path;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE FUNCTION categories_validate_tree() RETURNS TRIGGER AS $$
BEGIN
    PERFORM category_path(NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER categories_valid_tree
AFTER INSERT OR UPDATE OF parent ON categories
FOR EACH ROW EXECUTE FUNCTION categories_validate_tree();

CREATE TABLE products (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    thumbnail URL NOT NULL,
    gallery URL[] NOT NULL,
    -- Domain is NOT restricted to positive values only, as a special offer could for example give
    -- away a product for free (with limited uses).
    price TWOPOINT_UDEC NOT NULL CHECK (price > 0),
    overview TEXT NOT NULL,
    description TEXT NOT NULL,
    in_stock UINT NOT NULL DEFAULT 0,
    category INT NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    amount_per_unit AMOUNT,
    visible BOOLEAN NOT NULL DEFAULT TRUE,
    vendor INT NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    origin TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER products_creation_time
BEFORE INSERT OR UPDATE ON products
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE TRIGGER products_update_time
BEFORE UPDATE ON products
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE TABLE special_offers (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    members_only BOOLEAN NOT NULL DEFAULT FALSE,
    limit_per_customer POSITIVE_INT,
    valid_from TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Null: offer must be removed manually.
    valid_until TIMESTAMP CONSTRAINT end_after_start CHECK (valid_until IS NULL OR valid_until > valid_from),

    -- The lack of sum types is noticeable here. There are three variants:
    -- 1. "NEW PRICE `new_price`" (sale) has both quantities as `NULL`.
    -- 2. "TAKE `quantity1` PAY FOR `quantity2`" has `new_price` as `NULL`.
    -- 3. "TAKE `quantity1` PAY `new_price`" has `quantity2` as `NULL`.
    new_price TWOPOINT_UDEC,
    quantity1 INT CHECK (quantity1 IS NULL OR quantity1 > 1),
    quantity2 INT CHECK (quantity2 IS NULL OR quantity2 >= 1)
);

CREATE VIEW active_special_offers AS
SELECT *
FROM SPECIAL_OFFERS
WHERE valid_from < CURRENT_TIMESTAMP AND (valid_until IS NULL OR valid_until > CURRENT_TIMESTAMP);

-- There is no technical reason why there couldn't be several active special offers: the price
-- calculator would just have to choose the better price.
CREATE FUNCTION offers_deny_overlap() RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM special_offers
        WHERE id != NEW.id
        AND product = NEW.product
        AND tsrange(valid_from, valid_until, '[)') && tsrange(NEW.valid_from, NEW.valid_until, '[)')
    ) THEN
        RAISE EXCEPTION 'Attempted to create overlapping special offers. Consider deactivating one.';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER offers_no_overlap
BEFORE INSERT OR UPDATE OF product, valid_from, valid_until ON special_offers
FOR EACH ROW EXECUTE FUNCTION offers_deny_overlap();

CREATE FUNCTION offers_discount(
    base_price TWOPOINT_UDEC,
    new_price TWOPOINT_UDEC,
    quantity1 INT,
    quantity2 INT
) RETURNS DECIMAL AS $$
DECLARE
    discount DECIMAL;
BEGIN
    -- Variant 1.
    IF new_price IS NOT NULL AND quantity1 IS NULL AND quantity2 IS NULL THEN
        IF new_price >= base_price THEN
            RAISE EXCEPTION 'New price is not less than base price.';
        END IF;
        
        IF base_price = 0 THEN
            discount := 1;
        ELSE
            discount := 1 - new_price / base_price;
        END IF;
    -- Variant 2.
    ELSIF new_price IS NULL AND quantity1 IS NOT NULL AND quantity2 IS NOT NULL THEN
        IF quantity1 <= 1 THEN
            RAISE EXCEPTION 'Must be asked to take more than 1.';
        ELSIF quantity2 < 1 THEN
            RAISE EXCEPTION 'Must be asked to pay for at least 1.';
        ELSIF quantity1 <= quantity2 THEN
            RAISE EXCEPTION 'Must be asked to pay for less than taken.';
        END IF;
        discount := 1 - quantity2::TWOPOINT_UDEC / quantity1::TWOPOINT_UDEC;
    -- Variant 3.
    ELSIF new_price IS NOT NULL AND quantity1 IS NOT NULL AND quantity2 IS NULL THEN
        IF quantity1 <= 1 THEN
            RAISE EXCEPTION 'Must be asked to take more than 1.';
        ELSIF new_price >= base_price * quantity1 THEN
            RAISE EXCEPTION 'Must be asked to pay less in bulk.';
        END IF;
        
        IF base_price = 0 THEN
            discount := 1;
        ELSE
            discount := 1 - new_price / (base_price * quantity1);
        END IF;
    ELSE
        RAISE EXCEPTION 'Invalid variant.';
    END IF;

    RETURN discount;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE FUNCTION offers_validate_discount() RETURNS TRIGGER AS $$
DECLARE
    base_price TWOPOINT_UDEC;
BEGIN
    SELECT price INTO base_price FROM products WHERE id = NEW.product;
    PERFORM offers_discount(base_price, NEW.new_price, NEW.quantity1, NEW.quantity2);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER offers_valid_discount
BEFORE INSERT OR UPDATE OF product, new_price, quantity1, quantity2 ON special_offers
FOR EACH ROW EXECUTE FUNCTION offers_validate_discount();

CREATE FUNCTION products_validate_discounts() RETURNS TRIGGER AS $$
BEGIN
    PERFORM offers_discount(NEW.price, new_price, quantity1, quantity2)
    FROM special_offers so
    WHERE so.product = NEW.id AND (so.valid_until IS NULL OR so.valid_until > CURRENT_TIMESTAMP);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER products_valid_discounts
BEFORE UPDATE OF price ON products
FOR EACH ROW EXECUTE FUNCTION products_validate_discounts();

-- NOTE: It is possible for a customer to have used a special offer more times than the limit
-- allows due to the limit having changed. Similarly, it is possible for a non-member to have used
-- members-only special offer due to the status of the latter having changed. These are not errors
-- and nothing should be changed about the history, it should only prevent future uses.
CREATE TABLE special_offer_uses (
    special_offer INT NOT NULL REFERENCES special_offers(id) ON DELETE CASCADE,
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    count UINT NOT NULL DEFAULT 0,
    PRIMARY KEY (special_offer, customer)
);

-- NOTE: Tracks historical expiries as well. When products are sold, their next expiry should be
-- decremented and removed if 0.
CREATE TABLE expiries (
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    expiry DATE NOT NULL,
    amount POSITIVE_INT NOT NULL,
    -- Null: not processed yet.
    processed_at NONFUTURE_TIMESTAMP CONSTRAINT processed_after_expiry CHECK (processed_at >= expiry)
);

CREATE FUNCTION process_expiries() RETURNS void AS $$
    -- Key doesn't matter as long as it's unique.
    WITH lock AS (SELECT pg_advisory_xact_lock(hashtextextended('process_expiries'))),
    processed AS (
        UPDATE expiries
        SET processed_at = CURRENT_TIMESTAMP
        WHERE expiry <= CURRENT_DATE AND processed_at IS NULL
        RETURNING product, amount
    ),
    counts AS (
        SELECT product, SUM(amount) as count
        FROM processed
        GROUP BY product
    )
    UPDATE products
    -- We accept that there might have "disappeared" products due to manual intervention. Maybe some
    -- units arrived with broken packaging.
    SET in_stock = GREATEST(0, products.in_stock - counts.count)
    FROM counts
    WHERE products.id = counts.product;
$$ LANGUAGE sql VOLATILE;

-- WARN: Only actually runs at midnight. If the database is down at that time, expiries will be
-- missed. Hence, call this function on establishing a connection to the database. If this is done,
-- there will be no issues with data integrity as the downage would also prevent orders from being
-- placed.
-- Safer alternatives would be to have the database automatically call this at startup (is this
-- possible?) or to run it on each relevant access to affected tables (is this feasible?).
SELECT cron.schedule (
    'process_daily_expiries',
    -- Daily at midnight.
    '0 0 * * *',
    $$
    SELECT process_expiries();
    $$
);

-- Only customers are allowed to rate and review products. Vendors woulf use these only to inflate
-- scores on their own products, and administrators have no reason to. However, all users can reply
-- to reviews and comments, as they might want to answer questions or clear up confusions.

CREATE TABLE ratings (
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    rating RATING NOT NULL,
    PRIMARY KEY (product, customer)
);

CREATE TABLE reviews (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    CONSTRAINT one_review_per_customer_per_product UNIQUE (product, customer),
    -- Deleting their review is probably not what a customer intends when unsetting their rating.
    FOREIGN KEY (product, customer) REFERENCES ratings ON DELETE RESTRICT
);

CREATE TRIGGER reviews_creation_time
BEFORE INSERT OR UPDATE ON reviews
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE TRIGGER reviews_update_time
BEFORE UPDATE ON reviews
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE FUNCTION reviewer_can_review() RETURNS TRIGGER AS $$
BEGIN
    IF NOT (SELECT can_review FROM customers WHERE id = NEW.customer) THEN
        RAISE EXCEPTION 'Customer must be able to place reviews.';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_reviewer
BEFORE INSERT OR UPDATE OF customer ON reviews
FOR EACH ROW EXECUTE FUNCTION reviewer_can_review();

CREATE TABLE review_votes (
    review INT NOT NULL REFERENCES reviews(id) ON DELETE CASCADE,
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    grade VOTE NOT NULL
);

CREATE TABLE comments (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- We allow vendors (and administrators) to place comments, for example to respond to critique.
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Child comments also have this set for easier queries.
    review INT NOT NULL REFERENCES reviews(id) ON DELETE CASCADE,
    parent INT REFERENCES comments(id) ON DELETE CASCADE
);

CREATE TRIGGER comments_creation_time
BEFORE INSERT OR UPDATE ON comments
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE TRIGGER comments_update_time
BEFORE UPDATE ON comments
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE FUNCTION comment_parent_same_review() RETURNS TRIGGER AS $$
BEGIN

    IF NEW.parent IS NOT NULL
        AND NEW.review != (SELECT review FROM comments WHERE id = NEW.parent)
    THEN
        RAISE EXCEPTION 'Parent comment must belong to same review as all children.';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER comment_same_review
BEFORE INSERT OR UPDATE OF parent, review ON comments
FOR EACH ROW EXECUTE FUNCTION comment_parent_same_review();

CREATE FUNCTION comments_validate_tree() RETURNS TRIGGER AS $$
DECLARE
    visited INT[] := ARRAY[NEW.id];
    current_id INT := NEW.parent;
    current_parent INT;
BEGIN
    WHILE current_id IS NOT NULL LOOP
        IF current_id = ANY(visited) THEN
            RAISE EXCEPTION 'Cycle detected.';
        END IF;

        visited := visited || current_id;
        SELECT parent INTO current_parent FROM comments WHERE id = current_id;
        current_id := current_parent;
    END LOOP;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE TRIGGER comments_valid_tree
BEFORE INSERT OR UPDATE OF parent ON comments
FOR EACH ROW EXECUTE FUNCTION comments_validate_tree();

CREATE TABLE comment_votes (
    comment INT NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    grade VOTE NOT NULL
);

CREATE TABLE shopping_cart_items (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    -- Null: product was deleted since being added to cart. The customer can see that this has
    -- happened, but not what the product was.
    product INT REFERENCES products(id) ON DELETE SET NULL,
    count POSITIVE_INT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT one_item_per_customer_per_product UNIQUE (customer, product)
);

CREATE TRIGGER cart_update_time
BEFORE UPDATE ON shopping_cart_items
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE TABLE customer_favorites (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    favorited_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (customer, product)
);

CREATE TRIGGER favorites_creation_time
BEFORE INSERT OR UPDATE ON favorites
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE TABLE orders (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- It's realistic to require a complete log of purchases, so customers are only "soft deleted"
    -- by having their `deleted` column set to true.
    customer INT NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
    -- Restricting product deletion would practically require also restricting product
    -- modification, as changing the name, price, etc. of a product invalidates order logs as much
    -- as deleting it. The important part, that being the user and the price, is still kept. If
    -- proper audit logging is important, the product table needs a redesign.
    product INT REFERENCES products(id) ON DELETE SET NULL,

    -- At the time of purchase.
    price TWOPOINT_UDEC NOT NULL,
    amount_per_unit AMOUNT,
    count POSITIVE_INT NOT NULL,

    time NONFUTURE_TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- TODO: Indices.

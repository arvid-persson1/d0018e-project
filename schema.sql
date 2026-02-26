-- This is an open copy of the database schema. It is NOT included in any form of build script, CI
-- or any other form of validation, so changes here are not automatically reflected in the actual
-- database. Rather this is meant as a form of documentation of the database schema manually kept
-- up-to-date.

CREATE EXTENSION citext;
CREATE EXTENSION btree_gist;
CREATE EXTENSION pg_cron;

CREATE DOMAIN USERNAME AS TEXT CONSTRAINT valid_username CHECK (
    VALUE ~ '^[[:word:]-]{3,20}$'
);

-- NOTE: This is the HTML5 specification, specifically incompatible with RFC5322.
CREATE DOMAIN EMAIL AS citext CONSTRAINT valid_email CHECK (
    VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$'
);

CREATE DOMAIN NONFUTURE_TIMESTAMP AS TIMESTAMP CHECK (VALUE <= CURRENT_TIMESTAMP);

-- Arbitrary choice of precision and scale.
CREATE DOMAIN TWOPOINT_UDEC AS DECIMAL(10, 2) CHECK (VALUE >= 0);

-- TODO: Use more suitable integer types. For example, all current uses of `UINT` and
-- `POSITIVE_INT` would work just as well with a backing `SMALLINT`, while IDs in some of the
-- larger tables should possibly be `BIGINT`.

CREATE DOMAIN UINT AS INT CHECK (VALUE >= 0);

CREATE DOMAIN POSITIVE_INT AS INT CHECK (VALUE > 0);

CREATE DOMAIN RATING AS INT CHECK (VALUE BETWEEN 1 AND 5);

-- TODO: Improve URL representation or replace entirely (server storage).
CREATE DOMAIN URL AS TEXT;

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
    -- NOTE: There is no administrators table, since administrators do not need any information
    -- beyond what any user already has. For example, they share a common profile picture.
    role ROLE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- This column can't be automatically generated as it has to be written to by triggers when
    -- "subclass" is updated.
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE FUNCTION creation_time() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
BEGIN
    NEW.created_at = CASE TG_OP
        WHEN 'INSERT' THEN CURRENT_TIMESTAMP
        WHEN 'UPDATE' THEN OLD.created_at
    END;

    RETURN NEW;
END;
$$;

CREATE TRIGGER users_creation_time
BEFORE INSERT OR UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE FUNCTION update_time() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at := CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;

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
    display_name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL
);

CREATE FUNCTION update_time_user_super() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
    RETURN NEW;
END;
$$;

CREATE TRIGGER customers_update_time_super
BEFORE UPDATE ON customers
FOR EACH ROW EXECUTE FUNCTION update_time_user_super();

CREATE TRIGGER vendors_update_time_super
BEFORE UPDATE ON vendors
FOR EACH ROW EXECUTE FUNCTION update_time_user_super();

CREATE FUNCTION validate_user_subclass() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
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
$$;

CREATE CONSTRAINT TRIGGER users_valid_subclass
AFTER INSERT OR UPDATE OF role ON users
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_user_subclass();

CREATE FUNCTION validate_user_superclass() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
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
$$;

CREATE TRIGGER customers_valid_superclass
BEFORE INSERT OR UPDATE OF id ON customers
FOR EACH ROW EXECUTE FUNCTION validate_user_superclass();

CREATE TRIGGER vendors_valid_superclass
BEFORE INSERT OR UPDATE OF id ON vendors
FOR EACH ROW EXECUTE FUNCTION validate_user_superclass();

CREATE FUNCTION validate_user_role_change() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
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
$$;

CREATE TRIGGER users_valid_role_change
BEFORE UPDATE OF role ON users
FOR EACH ROW EXECUTE FUNCTION validate_user_role_change();

CREATE FUNCTION validate_user_subclass_deletion() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM users WHERE id = OLD.id) THEN
        RAISE EXCEPTION 'Must remove user superclass along with subclass.';
    END IF;
    RETURN OLD;
END;
$$;

CREATE TRIGGER customers_deletion
BEFORE DELETE ON customers
FOR EACH ROW EXECUTE FUNCTION validate_user_subclass_deletion();

CREATE TRIGGER vendors_deletion
BEFORE DELETE ON vendors
FOR EACH ROW EXECUTE FUNCTION validate_user_subclass_deletion();

CREATE FUNCTION validate_user_one_subclass() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
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
$$;

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

CREATE INDEX categories_by_parent_name ON categories (parent NULLS FIRST, name);

CREATE TYPE CATEGORY_PATH_SEGMENT AS (id INT, name TEXT);
CREATE FUNCTION category_path(start_id categories.id%TYPE) RETURNS category_path_segment[]
LANGUAGE plpgsql STABLE STRICT PARALLEL SAFE AS $$
DECLARE
    path category_path_segment[] := ARRAY[]::category_path_segment[];
    current_id categories.id%TYPE := start_id;
    current_name categories.name%TYPE;
    current_parent categories.parent%TYPE;
BEGIN
    LOOP
        SELECT id, name, parent
        INTO STRICT current_id, current_name, current_parent
        FROM categories WHERE id = current_id;
        IF EXISTS (SELECT 1 FROM unnest(path) WHERE id = current_id) THEN
            RAISE EXCEPTION 'Cycle detected.';
        END IF;

        path := path || (current_id, current_name)::CATEGORY_PATH_SEGMENT;

        IF current_parent IS NULL THEN
            EXIT;
        END IF;

        current_id := current_parent;
    END LOOP;

    RETURN path;
END;
$$;

CREATE FUNCTION categories_validate_tree() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN
    PERFORM category_path(NEW.id);
    RETURN NEW;
END;
$$;

CREATE TRIGGER categories_valid_tree
AFTER INSERT OR UPDATE OF parent ON categories
FOR EACH ROW EXECUTE FUNCTION categories_validate_tree();

CREATE TABLE products (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    thumbnail URL NOT NULL,
    gallery URL[] NOT NULL,
    price TWOPOINT_UDEC NOT NULL CHECK (price > 0),
    overview TEXT NOT NULL,
    description TEXT NOT NULL,
    in_stock UINT NOT NULL DEFAULT 0,
    visible BOOLEAN NOT NULL DEFAULT TRUE,
    vendor INT NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    category INT NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    origin TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    amount_per_unit TWOPOINT_UDEC NOT NULL DEFAULT 1,
    -- Null: discrete amount.
    measurement_unit TEXT,
    CONSTRAINT valid_without_unit CHECK (measurement_unit IS NOT NULL OR amount_per_unit % 1 = 0)
);

CREATE INDEX products_by_vendor ON products (vendor);
CREATE INDEX products_by_category ON products (category);
CREATE INDEX visible_products_by_time ON products (created_at DESC)
WHERE visible AND in_stock > 0;

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
    -- Measured in number of batches (per unit if there is no concept of a batch).
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
    quantity2 INT CHECK (quantity2 IS NULL OR quantity2 >= 1),

    -- There is no technical or logical reason why there couldn't be several active special offers:
    -- the  price calculator would just have to choose the better price.
    CONSTRAINT no_overlap EXCLUDE USING gist (
        product WITH =,
        tsrange(valid_from, valid_until) WITH &&
    )
);

-- Can't make partial to active offers as that depends on the time.
CREATE INDEX offers_by_product ON special_offers (product);

CREATE VIEW active_special_offers AS
SELECT *
FROM special_offers
WHERE valid_from < CURRENT_TIMESTAMP AND (valid_until IS NULL OR valid_until > CURRENT_TIMESTAMP);

CREATE FUNCTION average_discount(
    base_price products.price%TYPE,
    new_price special_offers.new_price%TYPE,
    quantity1 special_offers.quantity1%TYPE,
    quantity2 special_offers.quantity2%TYPE
) RETURNS TWOPOINT_UDEC
LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$
DECLARE
    discount TWOPOINT_UDEC;
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
$$;

CREATE FUNCTION offers_validate_discount() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
DECLARE
    base_price products.price%TYPE;
BEGIN
    SELECT price INTO STRICT base_price FROM products WHERE id = NEW.product;
    PERFORM average_discount(base_price, NEW.new_price, NEW.quantity1, NEW.quantity2);
    RETURN NEW;
END;
$$;

CREATE TRIGGER offers_valid_discount
BEFORE INSERT OR UPDATE OF product, new_price, quantity1, quantity2 ON special_offers
FOR EACH ROW EXECUTE FUNCTION offers_validate_discount();

CREATE FUNCTION products_validate_discounts() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN
    PERFORM average_discount(NEW.price, new_price, quantity1, quantity2)
    FROM special_offers so
    WHERE so.product = NEW.id AND (so.valid_until IS NULL OR so.valid_until > CURRENT_TIMESTAMP);
    RETURN NEW;
END;
$$;

CREATE TRIGGER products_valid_discounts
BEFORE UPDATE OF price ON products
FOR EACH ROW EXECUTE FUNCTION products_validate_discounts();

-- NOTE: It is possible for a customer to have used a special offer more times than the limit
-- allows due to the limit having changed. Similarly, it is possible for a non-member to have used
-- members-only special offer due to the status of the latter having changed. These are not errors
-- and nothing should be changed about the history, it should only prevent future uses.
CREATE TABLE special_offer_uses (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    special_offer INT NOT NULL REFERENCES special_offers(id) ON DELETE CASCADE,
    number UINT NOT NULL DEFAULT 0,
    PRIMARY KEY (special_offer, customer)
);

-- NOTE: Tracks historical expiries as well. When products are sold, their next expiry should be
-- decremented and removed if 0.
CREATE TABLE expiries (
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    expiry DATE NOT NULL,
    number POSITIVE_INT NOT NULL,
    -- Null: not processed yet.
    processed_at NONFUTURE_TIMESTAMP CONSTRAINT processed_after_expiry CHECK (processed_at >= expiry),
    CONSTRAINT aggregate_expiries UNIQUE (product, expiry)
);

CREATE INDEX expiries_by_product_processed ON expiries (product, processed_at NULLS FIRST);
CREATE INDEX expiries_by_product_date ON expiries (product, expiry);

CREATE VIEW pending_expiries AS
SELECT *
FROM expiries
WHERE processed_at IS NULL AND expiry <= CURRENT_DATE;

-- PERF: Not currently supported by an index.
CREATE FUNCTION process_expiries() RETURNS TABLE (
    product INT,
    -- NOTE: This is the naive sum of the listed number of expiries. It could be the case that
    -- stock has been reduced for reasons other than expiries or purchases, in which case this
    -- does *not* represent the number of products that actually expired.
    total BIGINT
) LANGUAGE sql VOLATILE AS $$
    WITH product_lock AS (
        SELECT id
        FROM products
        JOIN pending_expiries ON pending_expiries.product = products.id
        FOR UPDATE OF products
    ),
    processed AS (
        UPDATE pending_expiries
        SET processed_at = CURRENT_TIMESTAMP
        RETURNING product, number
    ),
    counts AS (
        SELECT product, SUM(number) AS total
        FROM processed
        GROUP BY product
    )
    UPDATE products
    -- We accept that there might have "disappeared" products due to manual intervention. Maybe some
    -- units arrived with broken packaging.
    SET in_stock = GREATEST(products.in_stock - counts.total, 0)
    FROM counts
    WHERE products.id = counts.product
    RETURNING products.id, counts.total
$$;

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

CREATE FUNCTION add_stock(
    product_id INT,
    number INT,
    expiry DATE = NULL
) RETURNS INT
LANGUAGE plpgsql AS $$
DECLARE
    new_stock INT;
BEGIN
    IF expiry IS NOT NULL THEN
        INSERT INTO expiries (product, expiry, number)
        VALUES (product_id, expiry, number)
        ON CONFLICT (product, expiry) DO UPDATE
        SET number = expiries.number + EXCLUDED.number;
    END IF;
    
    UPDATE products
    SET in_stock = in_stock + number
    WHERE id = product_id
    RETURNING in_stock INTO STRICT new_stock;

    RETURN new_stock;
END;
$$;

CREATE OR REPLACE FUNCTION sale_remove_expiries(
    product_id expiries.product%TYPE,
    number INT
) RETURNS INT
LANGUAGE plpgsql VOLATILE AS $$
DECLARE
    remaining INT := number;
    current_date expiries.expiry%TYPE;
    current_number expiries.number%TYPE;
BEGIN
    IF remaining < 0 THEN
        RAISE EXCEPTION 'Can''t sell negative number.';
    END IF;

    FOR current_date, current_number IN
        SELECT expiry, number
        FROM expiries
        WHERE product = product_id AND processed_at IS NULL
        ORDER BY expiry
        FOR UPDATE
    LOOP
        IF current_number > remaining THEN
            UPDATE expiries
            SET number = number - remaining
            WHERE product = product_id AND expiry = current_date;

            RETURN 0;
        ELSE
            remaining := remaining - current_number;
            DELETE FROM expiries
            WHERE product = product_id AND expiry = current_date;
        END IF;
    END LOOP;

    RETURN remaining;
END;
$$;

-- Only customers are allowed to rate and review products. Vendors woulf use these only to inflate
-- scores on their own products, and administrators have no reason to. However, all users can reply
-- to reviews and comments, as they might want to answer questions or clear up confusions.

CREATE TABLE ratings (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    rating RATING NOT NULL,
    PRIMARY KEY (product, customer)
);

CREATE FUNCTION rater_has_purchase() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM orders WHERE customer = NEW.customer AND product = NEW.product) THEN
        RAISE EXCEPTION 'Customer must have previously bought the product to rate it.';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER validate_rater
BEFORE INSERT OR UPDATE OF customer, product ON ratings
FOR EACH ROW EXECUTE FUNCTION rater_has_purchase();

CREATE TABLE reviews (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    CONSTRAINT one_review_per_customer_per_product UNIQUE (product, customer),
    -- Deleting their review is probably not what a customer intends when unsetting their rating.
    FOREIGN KEY (product, customer) REFERENCES ratings ON DELETE RESTRICT
);

CREATE INDEX reviews_by_customer_update ON reviews (customer, updated_at DESC);
CREATE INDEX reviews_by_product ON reviews (product);

CREATE TRIGGER reviews_creation_time
BEFORE INSERT OR UPDATE ON reviews
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE TRIGGER reviews_update_time
BEFORE UPDATE ON reviews
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE FUNCTION reviewer_can_review() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN
    IF NOT (SELECT can_review FROM customers WHERE id = NEW.customer) THEN
        RAISE EXCEPTION 'Customer must be able to place reviews.';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER validate_reviewer
BEFORE INSERT OR UPDATE OF customer ON reviews
FOR EACH ROW EXECUTE FUNCTION reviewer_can_review();

CREATE TABLE review_votes (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    review INT NOT NULL REFERENCES reviews(id) ON DELETE CASCADE,
    grade VOTE NOT NULL,
    PRIMARY KEY (review, customer)
);

CREATE FUNCTION no_vote_on_own_review() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN
    IF NEW.customer = (SELECT customer FROM reviews WHERE id = NEW.review) THEN
        RAISE EXCEPTION 'Can''t vote on own review.';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER deny_own_review_vote
BEFORE INSERT OR UPDATE OF customer, review ON review_votes
FOR EACH ROW EXECUTE FUNCTION no_vote_on_own_review();

CREATE TABLE comments (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- We allow vendors (and administrators) to place comments, for example to respond to critique.
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Child comments also have this set for easier queries.
    review INT NOT NULL REFERENCES reviews(id) ON DELETE CASCADE,
    -- Null: belongs to review directly.
    parent INT DEFAULT NULL REFERENCES comments(id) ON DELETE CASCADE
);

CREATE INDEX comments_by_review_parent ON comments (review, parent NULLS FIRST);

CREATE TRIGGER comments_creation_time
BEFORE INSERT OR UPDATE ON comments
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE TRIGGER comments_update_time
BEFORE UPDATE ON comments
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE FUNCTION comment_parent_same_review() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN

    IF NEW.parent IS NOT NULL
        AND NEW.review != (SELECT review FROM comments WHERE id = NEW.parent)
    THEN
        RAISE EXCEPTION 'Parent comment must belong to same review as all children.';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER comment_same_review
BEFORE INSERT OR UPDATE OF parent, review ON comments
FOR EACH ROW EXECUTE FUNCTION comment_parent_same_review();

CREATE FUNCTION comments_validate_tree() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
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

        SELECT parent 
        INTO STRICT current_parent
        FROM comments
        WHERE id = current_id;
        current_id := current_parent;
    END LOOP;
    
    RETURN NEW;
END;
$$;

CREATE TRIGGER comments_valid_tree
BEFORE INSERT OR UPDATE OF parent ON comments
FOR EACH ROW EXECUTE FUNCTION comments_validate_tree();

CREATE TABLE comment_votes (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    comment INT NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    grade VOTE NOT NULL,
    PRIMARY KEY (comment, customer)
);

CREATE FUNCTION no_vote_on_own_comment() RETURNS TRIGGER
LANGUAGE plpgsql STABLE AS $$
BEGIN
    IF NEW.customer = (SELECT customer FROM comments WHERE id = NEW.comment) THEN
        RAISE EXCEPTION 'Can''t vote on own comment.';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER deny_own_comment_vote
BEFORE INSERT OR UPDATE OF customer, comment ON comment_votes
FOR EACH ROW EXECUTE FUNCTION no_vote_on_own_comment();

CREATE TABLE shopping_cart_items (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    -- Null: product was deleted since being added to cart. The customer can see that this has
    -- happened, but not what the product was.
    product INT REFERENCES products(id) ON DELETE SET NULL,
    number POSITIVE_INT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Unique `(customer, product)` pairs except that `NULL = NULL` for `product`.
CREATE UNIQUE INDEX items_valid ON shopping_cart_items (customer, product)
WHERE product IS NOT NULL;
CREATE UNIQUE INDEX items_removed ON shopping_cart_items (customer)
WHERE product IS NULL;

CREATE TRIGGER cart_update_time
BEFORE UPDATE ON shopping_cart_items
FOR EACH ROW EXECUTE FUNCTION update_time();

CREATE PROCEDURE set_visibility(product_id INT, visible BOOLEAN)
LANGUAGE plpgsql AS $$
BEGIN
    IF NOT visible THEN
        UPDATE shopping_cart_items
        SET product = NULL
        WHERE product = product_id;
    END IF;

    UPDATE products
    SET visible = visible
    WHERE id = product_id;
END;
$$;

CREATE FUNCTION calculate_price(
    base_price products.price%TYPE,
    number shopping_cart_items.number%TYPE,
    new_price special_offers.new_price%TYPE,
    quantity1 special_offers.quantity1%TYPE,
    quantity2 special_offers.quantity2%TYPE,
    remaining_uses UINT,
    OUT price DECIMAL(10, 2),
    OUT uses INT
) LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$
BEGIN
    IF base_price IS NULL THEN
        RAISE EXCEPTION 'Base price must not be null.';
    ELSIF number IS NULL THEN
        RAISE EXCEPTION 'Number of units must not be null.';
    ELSIF remaining_uses IS NULL THEN
        RAISE EXCEPTION 'Remaining uses must not be null.';
    END IF;

    -- No special offer.
    IF new_price IS NULL AND quantity1 IS NULL AND quantity2 IS NULL THEN
        uses := 0;
        price := base_price * number;
    -- Variant 1.
    ELSIF new_price IS NOT NULL AND quantity1 IS NULL AND quantity2 IS NULL THEN
        uses := LEAST(number, remaining_uses);
        price := uses * (new_price - base_price) + base_price * number;
    -- Variant 2.
    ELSIF new_price IS NULL AND quantity1 IS NOT NULL AND quantity2 IS NOT NULL THEN
        uses := LEAST(number / quantity1, remaining_uses);
        price := base_price * (number - uses * (quantity1 - quantity2));
    -- Variant 3.
    ELSIF new_price IS NOT NULL AND quantity1 IS NOT NULL AND quantity2 IS NULL THEN
        uses := LEAST(number / quantity1, remaining_uses);
        price := new_price * uses + base_price * (number - quantity1 * uses);
    ELSE
        RAISE EXCEPTION 'Invalid variant.';
    END IF;
END;
$$;

CREATE TABLE customer_favorites (
    customer INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    product INT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (product, customer)
);

CREATE TRIGGER favorites_creation_time
BEFORE INSERT OR UPDATE ON customer_favorites
FOR EACH ROW EXECUTE FUNCTION creation_time();

CREATE PROCEDURE delete_user(id INT)
LANGUAGE plpgsql AS $$
BEGIN
    -- NOTE: Soft deletion. Possible corresponding row in role-specific table is also kept.
    UPDATE users
    SET deleted = true
    WHERE id = id;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'User does not exist.';
    END IF;

    -- PERF: Several of these queries are not supported by indices: we imagine account deletions
    -- are rare.
    IF EXISTS (SELECT 1 FROM customers WHERE id = id) THEN
        DELETE FROM special_offer_uses
        WHERE customer = id;

        -- NOTE: Reviews must be deleted before ratings.
        DELETE FROM reviews
        WHERE customer = id;

        DELETE FROM ratings
        WHERE customer = id;

        DELETE FROM review_votes
        WHERE customer = id;

        DELETE FROM comment_votes
        WHERE customer = id;

        DELETE FROM shopping_cart_items
        WHERE customer = id;

        DELETE FROM customer_favorites
        WHERE customer = id;
    ELSIF EXISTS (SELECT 1 FROM vendors WHERE id = id) THEN
        DELETE FROM products
        WHERE vendor = id;
    END IF;

    DELETE FROM comments
    WHERE user_id = id;
END;
$$;

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
    time NONFUTURE_TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- A more strict history could be maintained. For example, it might be desirable to store a
    -- copy of the deal from the special offers used (or at least the discount), and basic
    -- information in the case that the product is deleted.

    number POSITIVE_INT NOT NULL,
    paid TWOPOINT_UDEC NOT NULL,
    special_offer_used BOOLEAN NOT NULL,
    -- At the time of purchase.
    amount_per_unit TWOPOINT_UDEC NOT NULL DEFAULT 1,
    -- Null: discrete amount.
    measurement_unit TEXT,
    CONSTRAINT valid_without_unit CHECK (measurement_unit IS NOT NULL OR amount_per_unit % 1 = 0)
);

CREATE INDEX orders_per_customer_by_time ON orders (customer, time DESC);
CREATE INDEX orders_by_customer_product ON orders (customer, product);

CREATE PROCEDURE checkout(customer_id INT)
LANGUAGE plpgsql AS $$
DECLARE
    item_count INT;
BEGIN
    -- Can this be made into one statement or an "implicit" table creation?
    CREATE TEMP TABLE items (
        product INT,
        number POSITIVE_INT NOT NULL
    ) ON COMMIT DROP;
    WITH deleted AS (
        DELETE FROM shopping_cart_items
        WHERE customer = customer_id
        RETURNING product, number
    )
    INSERT INTO items
    SELECT product, number
    FROM deleted;

    -- Can this be inlined and the variable removed?
    GET DIAGNOSTICS item_count = ROW_COUNT;
    IF item_count = 0 THEN
        -- RAISE NOTICE 'Checkout with no items.';
        RETURN;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM items
        LEFT JOIN products ON id = product
        WHERE id IS NULL OR NOT visible
    ) THEN
        RAISE EXCEPTION 'Checkout with missing or invisible product.';
    END IF;

    UPDATE products
    -- Fails if negative.
    SET in_stock = in_stock - number
    FROM items
    WHERE id = product;

    PERFORM sale_remove_expiries(product, number)
    FROM items;

    CREATE TEMP TABLE results (
        product INT NOT NULL,
        number POSITIVE_INT NOT NULL,
        amount_per_unit TWOPOINT_UDEC NOT NULL,
        measurement_unit TEXT,
        price TWOPOINT_UDEC,
        uses INT NOT NULL,
        special_offer_id INT
    ) ON COMMIT DROP;
    INSERT INTO results
    SELECT i.product, i.number, amount_per_unit, measurement_unit, price, uses, aso.id
    FROM items i
    JOIN products ON products.id = product
    JOIN customers ON customers.id = customer_id
    LEFT JOIN active_special_offers aso ON aso.product = i.product
    LEFT JOIN special_offer_uses u ON u.special_offer = aso.id AND u.customer = customer_id
    CROSS JOIN LATERAL calculate_price(
        price,
        i.number,
        new_price,
        quantity1,
        quantity2,
        CASE
            WHEN members_only AND member_since IS NULL THEN 0
            ELSE GREATEST(limit_per_customer - COALESCE(u.number, 0), 0)
        END
    ) AS calc;

    INSERT INTO special_offer_uses (special_offer, customer, number)
    SELECT special_offer_id, customer_id, uses
    FROM results
    WHERE special_offer_id IS NOT NULL AND uses > 0
    ON CONFLICT (special_offer, customer) DO UPDATE
    SET uses = uses + EXCLUDED.uses;

    INSERT INTO orders (customer, product, number, paid, special_offer_used, amount_per_unit, measurement_unit)
    SELECT customer_id, product, number, price, uses > 0, amount_per_unit, measurement_unit
    FROM results;
END;
$$;

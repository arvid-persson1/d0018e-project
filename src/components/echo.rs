use dioxus::prelude::*;

/// Echo component for demonstration of server functions.
#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(String::new);

    rsx! {
        div { id: "echo",
            h1 { "Server echo" }
            input {
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

#[post("/api/echo")]
async fn echo_server(input: String) -> Result<String> {
    Ok(input)
}

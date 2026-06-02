pub fn get_telegram_client() -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder();

    let enabled = std::env::var("PROXY_ENABLED_TG")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if enabled {
        let host = std::env::var("PROXY_HOST").unwrap_or_default();
        let port = std::env::var("PROXY_PORT").unwrap_or_default();
        let user = std::env::var("PROXY_USER").ok();
        let pass = std::env::var("PROXY_PASSWORD").ok();

        if !host.is_empty() && !port.is_empty() {
            // Build socks5 proxy URL. If credentials are provided, embed them.
            let proxy_url = if let (Some(u), Some(p)) = (user, pass) {
                if !u.is_empty() && !p.is_empty() {
                    format!("socks5://{}:{}@{}:{}", u, p, host, port)
                } else {
                    format!("socks5://{}:{}", host, port)
                }
            } else {
                format!("socks5://{}:{}", host, port)
            };

            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(proxy);
                println!("SOCKS5 proxy configured for Telegram API: socks5://{}:{}", host, port);
            } else {
                eprintln!("Failed to parse proxy URL: {}", proxy_url);
            }
        }
    }

    builder.build()
}

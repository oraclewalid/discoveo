struct Config {
    google_client_id: String,
    google_client_secret: String,
    google_redirect_url: String,
    database_url: String,
}

impl Config {
    pub fn new() -> Self {
        let google_client_id = env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set");
        let google_client_secret =
            env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set");
        let google_redirect_url =
            env::var("GOOGLE_REDIRECT_URL").expect("GOOGLE_REDIRECT_URL must be set");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        Self {
            google_client_id,
            google_client_secret,
            google_redirect_url,
            database_url,
        }
    }
}

pub struct PasswordEncoder {}

impl PasswordEncoder {
    pub fn encode(raw_password: &str) -> String {
        let digest = md5::compute(raw_password);
        format!("{:x}", digest)
    }
    pub fn verify(password: &str, raw_password: &str) -> bool {
        if password.eq(raw_password) {
            return true;
        }
        let hashed = PasswordEncoder::encode(raw_password);
        password.eq(&hashed)
    }
}

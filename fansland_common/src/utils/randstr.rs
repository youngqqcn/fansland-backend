
use rand::Rng;

fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;,./<>?";
    let charset_len = CHARSET.len();

    let mut rng = rand::thread_rng();
    let random_string: String = (0..length)
        .map(|_| {
            let index = rng.gen_range(0, charset_len);
            CHARSET[index] as char
        })
        .collect();

    random_string
}
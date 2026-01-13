global MAX_URL_LEN: u32 = 20000;

fn main(urls: [[u8; MAX_URL_LEN]; 2], allowed_url: [u8; MAX_URL_LEN], allowed_url_index: u32) {
    assert(urls[allowed_url_index] == allowed_url);
}

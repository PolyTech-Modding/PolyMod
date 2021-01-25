use rand::Rng;

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn create() -> String {
    let size = 7;
    let mut id = String::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        id.push(BASE62[rng.gen::<usize>() % 62] as char);
    }

    id
}

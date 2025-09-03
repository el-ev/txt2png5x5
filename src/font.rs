const FONT_DATA: &[(u8, u32)] = &[
    (b'A', 0x00e8fe31),
    (b'B', 0x01e8fa3e),
    (b'C', 0x00f8420f),
    (b'D', 0x01e8c63e),
    (b'E', 0x01f8721f),
    (b'F', 0x01f87210),
    (b'G', 0x00f85e2f),
    (b'H', 0x0118fe31),
    (b'I', 0x01f2109f),
    (b'J', 0x01f0862e),
    (b'K', 0x01197251),
    (b'L', 0x0108421f),
    (b'M', 0x011dd631),
    (b'N', 0x011cd671),
    (b'O', 0x00e8c62e),
    (b'P', 0x01e8fa10),
    (b'Q', 0x00e8d66f),
    (b'R', 0x01e8fa51),
    (b'S', 0x00f8383e),
    (b'T', 0x01f21084),
    (b'U', 0x0118c62e),
    (b'V', 0x0118c544),
    (b'W', 0x0118d771),
    (b'X', 0x01151151),
    (b'Y', 0x01151084),
    (b'Z', 0x01f1111f),
    (b' ', 0x00000000),
    (b'!', 0x00421004),
    (b'?', 0x00e88884),
    (b'.', 0x00000004),
    (b',', 0x00000088),
    (b':', 0x00020080),
    (b';', 0x00020088),
    (b'\'', 0x00420000),
    (b'"', 0x00a50000),
    (b'-', 0x00007c00),
    (b'0', 0x00e9d72e),
    (b'1', 0x0046109f),
    (b'2', 0x00e8889f),
    (b'3', 0x00e89a2e),
    (b'4', 0x00232be2),
    (b'5', 0x01f8783e),
    (b'6', 0x00e87a2e),
    (b'7', 0x01f08888),
    (b'8', 0x00e8ba2e),
    (b'9', 0x00e8bc2e),
    (b'(', 0x00222082),
    (b')', 0x00820888),
];

pub const CHAR_WIDTH: u32 = 5;
pub const CHAR_HEIGHT: u32 = 5;

pub fn get_char_bitmap(c: u8) -> u32 {
    for &(ch, bitmap) in FONT_DATA {
        if ch == c {
            return bitmap;
        }
    }
    0
}

pub fn get_pixel(bitmap: u32, x: u32, y: u32) -> bool {
    let bit_index = (CHAR_HEIGHT - 1 - y) * CHAR_WIDTH + (CHAR_WIDTH - 1 - x);
    (bitmap & (1 << bit_index)) != 0
}

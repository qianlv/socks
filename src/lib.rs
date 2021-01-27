pub mod buffer;
pub mod sock4;
use mio::Token;

static mut NEXT_TOKEN: usize = 0;

pub unsafe fn next_token() -> Token {
    NEXT_TOKEN += 1;
    Token(NEXT_TOKEN)
}

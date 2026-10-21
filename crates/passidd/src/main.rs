use passid::lockout::{Gate, LockoutState};
use passid::passcode;

fn main() {
    let phc = passcode::hash("482915").expect("hash");
    println!("stored: {phc}\n");

    let mut state = LockoutState::default();

    let guesses = [
        "000000", "111222", "482916", "482917", "482918", "482919", "482915",
    ];

    for (now, guess) in (1_000u64..).zip(guesses) {
        match state.check(now) {
            Gate::Open => {
                if passcode::verify(guess, &phc) {
                    println!("{guess}: OK");
                    state.record_success();
                } else {
                    state.record_failure(now);
                    println!("{guess}: wrong (failures={})", state.failures);
                }
            }
            Gate::Wait(seconds) => {
                println!("{guess}: locked, wait {seconds}s");
            }
            Gate::FallbackRequired => {
                println!("{guess}: use password/passkey");
            }
        }
    }
}

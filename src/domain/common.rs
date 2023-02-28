use crate::error::{VortoError, VortoResult};


pub fn validate_fn(is_failed: impl Fn() -> bool, vorto_error: VortoError) -> VortoResult<()> {
    if is_failed() {
        VortoResult::Err(vorto_error)
    } else {
        VortoResult::Ok(())
    }
}
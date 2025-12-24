
#[cfg(test)]
mod tests {
    use rivus_axum::code::Code;
    use rivus_axum::r::R;

    #[test]
    fn test_r_ok() {
        let r = R::ok(123);
        assert_eq!(r.code, Code::Ok.as_i32());
        assert_eq!(r.message, "ok".to_string());
        assert_eq!(r.data, Some(123));
    }

    #[test]
    fn test_r_ok_with_message() {
        let r = R::ok_with_message("data".to_string(), "msg".to_string());
        assert_eq!(r.code, Code::Ok.as_i32());
        assert_eq!(r.message, "msg".to_string());
        assert_eq!(r.data, Some("data".to_string()));
    }

    #[test]
    fn test_r_err() {
        let r: R<()> = R::err(Code::BadRequest.as_i32());
        assert_eq!(r.code, Code::BadRequest.as_i32());
        assert_eq!(r.message, "error".to_string());
        assert_eq!(r.data, None);
    }

    #[test]
    fn test_r_err_with_message() {
        let r: R<()> = R::err_with_message(Code::Forbidden.as_i32(), "denied".to_string());
        assert_eq!(r.code, Code::Forbidden.as_i32());
        assert_eq!(r.message, "denied".to_string());
        assert_eq!(r.data, None);
    }
}
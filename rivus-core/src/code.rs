// 封装返回结果
#[repr(i32)]
#[derive(Copy, Clone)]
pub enum Code {
    // 成功：服务器成功接收客户端请求
    Ok = 200,

    // 请求错误：服务器无法理解客户端的请求
    BadRequest = 400,

    // 未认证：客户端未通过身份验证
    Unauthorized = 401,

    // 禁止访问：客户端没有访问内容的权限
    Forbidden = 403,

    // 未找到：服务器无法找到请求的资源
    NotFound = 404,

    // 请求过多：流量控制限制
    MethodNotAllowed = 405,

    // 请求过多：流量控制限制
    TooManyRequests = 429,

    // 身份验证错误：Token 或 AppKey 已过期
    IdentifyError = 430,

    // 身份验证过期：认证信息已过期
    IdentifyExpired = 431,

    // 签名错误：请求签名验证失败
    SignError = 432,

    // 服务器错误：服务器遇到错误，无法完成请求
    InternalServerError = 500,

    // 文件过大：超出最大允许上传文件大小
    FileTooLarge = 800,

    // 缺少必要请求头：请求中缺少必要头部字段
    MissingHeader = 900,

    // 参数不合法：客户端请求包含非法参数
    IllegalParam = 901,
}

impl Code {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_i32())
    }
}

#[test]
fn test_code() {
    assert_eq!(Code::Ok.as_i32(), 200);
    assert_eq!(Code::BadRequest.as_i32(), 400);
    assert_eq!(Code::Ok.to_string(), "200");
    assert_eq!(format!("{}", Code::InternalServerError), "500");
}

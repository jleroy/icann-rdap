use {
    super::{GtldParams, ToGtldWhois},
    icann_rdap_common::response::Common,
};

impl ToGtldWhois for Common {
    fn to_gtld_whois(&self, _params: &mut GtldParams) -> String {
        String::new()
    }
}

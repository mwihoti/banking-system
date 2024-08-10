#[cfg(test)]
mod tests {
    use super ::*;

    #[test]
    /// valid account numbers in different lengths
    fn valid_account_numbers_in_different_lengths() {
        let accounts = [
            AccountNumber::from_str("35001576202"),
            AccountNumber::from_str("8536276945"),
            AccountNumber::from_str("20024"),
            AccountNumber::from_str("26"),
        ];

        for account in accounts {
            assert! (account.is_ok());
        }
    }

    #[test]
    fn valid_account_numbers_fixed_length() {
        let valid_account_numbers = [
            "2334841596",
            "5072686164",
            "8330789085",
            "2303133926",
            "7730632457",
            "1310866767",
            "9083062142",
            "8936042657",
            "3188178648",
            "1513312791",
            "0204434294",
        ];
        for v in valid_account_numbers {
            let account = AccountNumber::from_str(v);
            assert!(account.is_ok())
        }
    }

    #[test]
    fn invalid_account_numbers_fixed_length() {
        let invalid_account_numbers = [
            "2334841592",
            "5072686163",
            "8330789084",
            "2303133925",
            "7730632456",
            "1310866766",
            "9083062141",
            "8936042656",
            "3188178647",
            "1513312790",
            "0204434293",
        ];
        for v in invalid_account_numbers {
            let account = AccountNumber::from_str(v);
            assert!(account.is_err());
        }
    }
}
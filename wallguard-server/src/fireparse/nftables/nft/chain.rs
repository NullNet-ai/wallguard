use crate::fireparse::nftables::{
    nft,
    token_stream::{Token, TokenStream},
};
use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub struct Chain {
    pub name: String,
    pub r#type: Option<nft::ChainType>,
    pub priority: Option<nft::Priority>,
    pub policy: Option<nft::Policy>,
    pub hook: Option<nft::Hook>,
    pub device: Option<String>,
}

impl From<String> for Chain {
    fn from(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

impl TryFrom<&mut TokenStream> for Chain {
    type Error = String;

    fn try_from(stream: &mut TokenStream) -> Result<Self, Self::Error> {
        stream.expect_ident("chain")?;

        let name = stream.expect_any_ident()?;
        let mut chain = Chain::from(name);

        stream.expect_symbol('{')?;
        stream.expect_newline()?;

        match stream.peek() {
            Some(Token::Ident(ident)) => {
                if ident == "type" {
                    let r#type = {
                        stream.expect_ident("type")?;
                        let value = stream.expect_any_ident()?;
                        nft::ChainType::from_str(&value)?
                    };

                    let hook = {
                        stream.expect_ident("hook")?;
                        let value = stream.expect_any_ident()?;
                        nft::Hook::from_str(&value)?
                    };

                    let device = {
                        match stream.peek() {
                            Some(Token::Ident(ident)) => {
                                if ident == "device" {
                                    stream.expect_ident("device")?;
                                    Some(stream.expect_any_ident()?)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    };

                    let priority = {
                        stream.expect_ident("priority")?;
                        let value = stream.expect_any_ident()?;
                        stream.expect_symbol(';')?;
                        nft::Priority::from_str(&value)?
                    };

                    let policy = {
                        match stream.peek() {
                            Some(Token::Ident(_)) => {
                                stream.expect_ident("policy")?;
                                let value = stream.expect_any_ident()?;
                                stream.expect_symbol(';')?;
                                Some(nft::Policy::from_str(&value)?)
                            }
                            Some(Token::Newline) => None,
                            Some(Token::Symbol(symb)) => Err(format!(
                                "Unexpected token. Expected NewLine or policy, got symbol {symb}"
                            ))?,
                            None => Err("Unepected end of stream")?,
                        }
                    };

                    chain.r#type = Some(r#type);
                    chain.hook = Some(hook);
                    chain.device = device;
                    chain.priority = Some(priority);
                    chain.policy = policy;
                }

                // Parse Matches & Statements

                // Just goto the end of the chain for now
                stream.skip_until(&Token::Symbol('}'));
                stream.rewind(stream.position() - 1);
            }
            // Empty Chain
            Some(Token::Symbol(_) | Token::Newline) => {}
            None => Err("Unexpected end of stream")?,
        }

        stream.expect_symbol('}')?;
        stream.expect_newline()?;

        Ok(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chain_empty() {
        let input = r#"
            chain INPUT {
            }
        "#;

        let mut stream = TokenStream::from(input);

        let chain = Chain::try_from(&mut stream).unwrap();

        assert_eq!(chain.name, "INPUT");
        assert!(chain.r#type.is_none());
        assert!(chain.priority.is_none());
        assert!(chain.policy.is_none());
        assert!(chain.hook.is_none());
        assert!(chain.device.is_none());
    }

    #[test]
    fn test_parse_chain_required_type_hook_priority() {
        let input = r#"
            chain INPUT {
                type filter hook input priority 0;
            }
        "#;

        let mut stream = TokenStream::from(input);

        println!("{:?}", stream);

        let chain = Chain::try_from(&mut stream).unwrap();

        assert_eq!(chain.name, "INPUT");
        assert_eq!(chain.r#type.unwrap(), nft::ChainType::Filter);
        assert_eq!(chain.priority.unwrap().get_value(), 0);
        assert_eq!(chain.hook.unwrap(), nft::Hook::Input);
        assert!(chain.policy.is_none());
        assert!(chain.device.is_none());
    }

    #[test]
    fn test_parse_chain_with_type_hook_priority_device() {
        let input = r#"
            chain INPUT {
                type filter hook input device eth0 priority 0;
            }
        "#;

        let mut stream = TokenStream::from(input);

        println!("{:?}", stream);

        let chain = Chain::try_from(&mut stream).unwrap();

        assert_eq!(chain.name, "INPUT");
        assert_eq!(chain.r#type.unwrap(), nft::ChainType::Filter);
        assert_eq!(chain.priority.unwrap().get_value(), 0);
        assert_eq!(chain.hook.unwrap(), nft::Hook::Input);
        assert_eq!(chain.device.unwrap(), "eth0");
        assert!(chain.policy.is_none());
    }

    #[test]
    fn test_parse_chain_with_type_hook_priority_policy() {
        let input = r#"
            chain INPUT {
                type filter hook input priority 0; policy accept;
            }
        "#;

        let mut stream = TokenStream::from(input);

        println!("{:?}", stream);

        let chain = Chain::try_from(&mut stream).unwrap();

        assert_eq!(chain.name, "INPUT");
        assert_eq!(chain.r#type.unwrap(), nft::ChainType::Filter);
        assert_eq!(chain.priority.unwrap().get_value(), 0);
        assert_eq!(chain.hook.unwrap(), nft::Hook::Input);
        assert!(chain.device.is_none());
        assert_eq!(chain.policy.unwrap(), nft::Policy::Accept);
    }

    #[test]
    fn test_parse_chain_with_type_hook_priority_device_policy() {
        let input = r#"
            chain INPUT {
                type filter hook input device eth0 priority 0; policy accept;
            }
        "#;

        let mut stream = TokenStream::from(input);

        println!("{:?}", stream);

        let chain = Chain::try_from(&mut stream).unwrap();

        assert_eq!(chain.name, "INPUT");
        assert_eq!(chain.r#type.unwrap(), nft::ChainType::Filter);
        assert_eq!(chain.priority.unwrap().get_value(), 0);
        assert_eq!(chain.hook.unwrap(), nft::Hook::Input);
        assert_eq!(chain.device.unwrap(), "eth0");
        assert_eq!(chain.policy.unwrap(), nft::Policy::Accept);
    }
}

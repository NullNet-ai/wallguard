use std::str::FromStr;

use crate::fireparse::nftables::{
    nft::{self},
    token_stream::{Token, TokenStream},
};

#[derive(Debug, Clone)]
pub struct Table {
    name: String,
    family: nft::Family,
    chains: Vec<nft::Chain>,
}

impl Table {
    pub fn new(name: String, family: nft::Family) -> Self {
        Self {
            name,
            family,
            chains: Default::default(),
        }
    }
}

impl TryFrom<&mut TokenStream> for Table {
    type Error = String;

    fn try_from(stream: &mut TokenStream) -> Result<Self, Self::Error> {
        stream.expect_ident("table")?;

        let family = {
            let ident = stream.expect_any_ident()?;
            nft::Family::from_str(&ident)?
        };

        let name = stream.expect_any_ident()?;

        stream.expect_symbol('{')?;
        stream.expect_newline()?;

        let mut table = Table::new(name, family);

        loop {
            match stream.peek() {
                Some(Token::Ident(ident)) if ident == "chain" => {
                    let chain = nft::Chain::try_from(&mut *stream)?;
                    table.chains.push(chain);
                }
                _ => break,
            }
        }

        stream.expect_symbol('}')?;
        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table() {
        let input = r#"
            table ip6 filter {
                    chain DOCKER {
                    }

                    chain DOCKER-FORWARD {
                        counter packets 0 bytes 0 jump DOCKER-CT
                        counter packets 0 bytes 0 jump DOCKER-ISOLATION-STAGE-1
                        counter packets 0 bytes 0 jump DOCKER-BRIDGE
                    }

                    chain DOCKER-BRIDGE {
                    }

                    chain DOCKER-CT {
                    }

                    chain DOCKER-ISOLATION-STAGE-1 {
                    }

                    chain DOCKER-ISOLATION-STAGE-2 {
                    }

                    chain FORWARD {
                        type filter hook forward priority filter; policy accept;
                        counter packets 0 bytes 0 jump DOCKER-USER
                        counter packets 0 bytes 0 jump DOCKER-FORWARD
                    }

                    chain DOCKER-USER {
                        counter packets 0 bytes 0 return
                    }
            }
        "#;

        let mut stream = TokenStream::from(input);

        let table = Table::try_from(&mut stream).unwrap();

        assert_eq!(table.name, "filter");
        assert_eq!(table.family, nft::Family::Ip6);

        assert_eq!(table.chains.len(), 8);
        assert_eq!(table.chains[0].name, "DOCKER");
        assert_eq!(table.chains[1].name, "DOCKER-FORWARD");
        assert_eq!(table.chains[2].name, "DOCKER-BRIDGE");
        assert_eq!(table.chains[3].name, "DOCKER-CT");
        assert_eq!(table.chains[4].name, "DOCKER-ISOLATION-STAGE-1");
        assert_eq!(table.chains[5].name, "DOCKER-ISOLATION-STAGE-2");
        assert_eq!(table.chains[6].name, "FORWARD");
        assert_eq!(table.chains[7].name, "DOCKER-USER");
    }
}

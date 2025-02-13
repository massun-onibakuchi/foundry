use crate::U256;
use alloy_primitives::U64;
use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};

pub use ethers_core::types::Chain as NamedChain;

/// Either a named or chain id or the actual id value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum Chain {
    /// Contains a known chain
    #[serde(serialize_with = "super::from_str_lowercase::serialize")]
    Named(NamedChain),
    /// Contains the id of a chain
    Id(u64),
}

impl Chain {
    /// The id of the chain.
    pub const fn id(&self) -> u64 {
        match self {
            Chain::Named(chain) => *chain as u64,
            Chain::Id(id) => *id,
        }
    }

    /// Returns the wrapped named chain or tries converting the ID into one.
    pub fn named(&self) -> Result<NamedChain> {
        match self {
            Self::Named(chain) => Ok(*chain),
            Self::Id(id) => {
                NamedChain::try_from(*id).map_err(|_| eyre::eyre!("Unsupported chain: {id}"))
            }
        }
    }

    /// Helper function for checking if a chainid corresponds to a legacy chainid
    /// without eip1559
    pub fn is_legacy(&self) -> bool {
        self.named().map_or(false, |c| c.is_legacy())
    }

    /// Returns the corresponding etherscan URLs
    pub fn etherscan_urls(&self) -> Option<(&'static str, &'static str)> {
        self.named().ok().and_then(|c| c.etherscan_urls())
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chain::Named(chain) => chain.fmt(f),
            Chain::Id(id) => {
                if let Ok(chain) = NamedChain::try_from(*id) {
                    chain.fmt(f)
                } else {
                    id.fmt(f)
                }
            }
        }
    }
}

impl From<NamedChain> for Chain {
    fn from(id: NamedChain) -> Self {
        Chain::Named(id)
    }
}

impl From<u64> for Chain {
    fn from(id: u64) -> Self {
        NamedChain::try_from(id).map(Chain::Named).unwrap_or_else(|_| Chain::Id(id))
    }
}

impl From<U256> for Chain {
    fn from(id: U256) -> Self {
        id.to::<u64>().into()
    }
}

impl From<Chain> for u64 {
    fn from(c: Chain) -> Self {
        match c {
            Chain::Named(c) => c as u64,
            Chain::Id(id) => id,
        }
    }
}

impl From<Chain> for U64 {
    fn from(c: Chain) -> Self {
        U64::from(u64::from(c))
    }
}

impl From<Chain> for U256 {
    fn from(c: Chain) -> Self {
        U256::from(u64::from(c))
    }
}

impl TryFrom<Chain> for NamedChain {
    type Error = <NamedChain as TryFrom<u64>>::Error;

    fn try_from(chain: Chain) -> Result<Self, Self::Error> {
        match chain {
            Chain::Named(chain) => Ok(chain),
            Chain::Id(id) => id.try_into(),
        }
    }
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(chain) = NamedChain::from_str(s) {
            Ok(Chain::Named(chain))
        } else {
            s.parse::<u64>()
                .map(Chain::Id)
                .map_err(|_| format!("Expected known chain or integer, found: {s}"))
        }
    }
}

impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ChainId {
            Named(String),
            Id(u64),
        }

        match ChainId::deserialize(deserializer)? {
            ChainId::Named(s) => {
                s.to_lowercase().parse().map(Chain::Named).map_err(serde::de::Error::custom)
            }
            ChainId::Id(id) => {
                Ok(NamedChain::try_from(id).map(Chain::Named).unwrap_or_else(|_| Chain::Id(id)))
            }
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        NamedChain::Mainnet.into()
    }
}

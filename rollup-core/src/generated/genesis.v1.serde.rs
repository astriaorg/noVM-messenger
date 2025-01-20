impl serde::Serialize for Account {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address.is_some() {
            len += 1;
        }
        if self.balance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("genesis.v1.Account", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if let Some(v) = self.balance.as_ref() {
            struct_ser.serialize_field("balance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Account {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["address", "balance"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Balance,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "address" => Ok(GeneratedField::Address),
                            "balance" => Ok(GeneratedField::Balance),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Account;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct genesis.v1.Account")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Account, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut balance__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::Balance => {
                            if balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balance"));
                            }
                            balance__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Account {
                    address: address__,
                    balance: balance__,
                })
            }
        }
        deserializer.deserialize_struct("genesis.v1.Account", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisAppState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.rollup_name.is_empty() {
            len += 1;
        }
        if self.sequencer_genesis_block_height != 0 {
            len += 1;
        }
        if self.celestia_genesis_block_height != 0 {
            len += 1;
        }
        if self.celestia_block_variance != 0 {
            len += 1;
        }
        if !self.accounts.is_empty() {
            len += 1;
        }
        if self.authority_sudo_address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("genesis.v1.GenesisAppState", len)?;
        if !self.rollup_name.is_empty() {
            struct_ser.serialize_field("rollupName", &self.rollup_name)?;
        }
        if self.sequencer_genesis_block_height != 0 {
            struct_ser.serialize_field(
                "sequencerGenesisBlockHeight",
                &self.sequencer_genesis_block_height,
            )?;
        }
        if self.celestia_genesis_block_height != 0 {
            struct_ser.serialize_field(
                "celestiaGenesisBlockHeight",
                &self.celestia_genesis_block_height,
            )?;
        }
        if self.celestia_block_variance != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field(
                "celestiaBlockVariance",
                ToString::to_string(&self.celestia_block_variance).as_str(),
            )?;
        }
        if !self.accounts.is_empty() {
            struct_ser.serialize_field("accounts", &self.accounts)?;
        }
        if let Some(v) = self.authority_sudo_address.as_ref() {
            struct_ser.serialize_field("authoritySudoAddress", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisAppState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rollup_name",
            "rollupName",
            "sequencer_genesis_block_height",
            "sequencerGenesisBlockHeight",
            "celestia_genesis_block_height",
            "celestiaGenesisBlockHeight",
            "celestia_block_variance",
            "celestiaBlockVariance",
            "accounts",
            "authority_sudo_address",
            "authoritySudoAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RollupName,
            SequencerGenesisBlockHeight,
            CelestiaGenesisBlockHeight,
            CelestiaBlockVariance,
            Accounts,
            AuthoritySudoAddress,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rollupName" | "rollup_name" => Ok(GeneratedField::RollupName),
                            "sequencerGenesisBlockHeight" | "sequencer_genesis_block_height" => {
                                Ok(GeneratedField::SequencerGenesisBlockHeight)
                            }
                            "celestiaGenesisBlockHeight" | "celestia_genesis_block_height" => {
                                Ok(GeneratedField::CelestiaGenesisBlockHeight)
                            }
                            "celestiaBlockVariance" | "celestia_block_variance" => {
                                Ok(GeneratedField::CelestiaBlockVariance)
                            }
                            "accounts" => Ok(GeneratedField::Accounts),
                            "authoritySudoAddress" | "authority_sudo_address" => {
                                Ok(GeneratedField::AuthoritySudoAddress)
                            }
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisAppState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct genesis.v1.GenesisAppState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisAppState, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut rollup_name__ = None;
                let mut sequencer_genesis_block_height__ = None;
                let mut celestia_genesis_block_height__ = None;
                let mut celestia_block_variance__ = None;
                let mut accounts__ = None;
                let mut authority_sudo_address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RollupName => {
                            if rollup_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rollupName"));
                            }
                            rollup_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SequencerGenesisBlockHeight => {
                            if sequencer_genesis_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "sequencerGenesisBlockHeight",
                                ));
                            }
                            sequencer_genesis_block_height__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                        GeneratedField::CelestiaGenesisBlockHeight => {
                            if celestia_genesis_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "celestiaGenesisBlockHeight",
                                ));
                            }
                            celestia_genesis_block_height__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                        GeneratedField::CelestiaBlockVariance => {
                            if celestia_block_variance__.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "celestiaBlockVariance",
                                ));
                            }
                            celestia_block_variance__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                        GeneratedField::Accounts => {
                            if accounts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accounts"));
                            }
                            accounts__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthoritySudoAddress => {
                            if authority_sudo_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "authoritySudoAddress",
                                ));
                            }
                            authority_sudo_address__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GenesisAppState {
                    rollup_name: rollup_name__.unwrap_or_default(),
                    sequencer_genesis_block_height: sequencer_genesis_block_height__
                        .unwrap_or_default(),
                    celestia_genesis_block_height: celestia_genesis_block_height__
                        .unwrap_or_default(),
                    celestia_block_variance: celestia_block_variance__.unwrap_or_default(),
                    accounts: accounts__.unwrap_or_default(),
                    authority_sudo_address: authority_sudo_address__,
                })
            }
        }
        deserializer.deserialize_struct("genesis.v1.GenesisAppState", FIELDS, GeneratedVisitor)
    }
}

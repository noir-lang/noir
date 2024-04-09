struct GasOpts {
  l1_gas: Option<Field>,
  l2_gas: Option<Field>,
  da_gas: Option<Field>,
}

impl GasOpts {
    pub fn default() -> Self {
        GasOpts { l1_gas: Option::none(), l2_gas: Option::none(), da_gas: Option::none() }
    }

    pub fn new(l1_gas: Field, l2_gas: Field, da_gas: Field) -> Self {
        GasOpts { l1_gas: Option::some(l1_gas), l2_gas: Option::some(l2_gas), da_gas: Option::some(da_gas) }
    }
}

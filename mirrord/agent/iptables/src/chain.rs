use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use crate::{
    error::{IPTablesError, IPTablesResult},
    IPTables,
};

#[derive(Debug)]
pub struct IPTableChain<IPT: IPTables> {
    inner: Arc<IPT>,
    chain_name: String,
    chain_size: AtomicI32,
}

impl<IPT> IPTableChain<IPT>
where
    IPT: IPTables,
{
    pub fn create(inner: Arc<IPT>, chain_name: String) -> IPTablesResult<Self> {
        inner.create_chain(&chain_name)?;

        // Start with 1 because the chain will always have at least `-A <chain name>` as a rule
        let chain_size = AtomicI32::from(1);

        Ok(IPTableChain {
            inner,
            chain_name,
            chain_size,
        })
    }

    pub fn load(inner: Arc<IPT>, chain_name: String) -> IPTablesResult<Self> {
        let existing_rules = inner.list_rules(&chain_name)?.len();

        if existing_rules == 0 {
            return Err(IPTablesError(
                format!("Unable to load rules for chain {chain_name}").into(),
            ));
        }

        // Start with 1 because the chain will allways have atleast `-A <chain name>` as a rule
        let chain_size = AtomicI32::from((existing_rules - 1) as i32);

        Ok(IPTableChain {
            inner,
            chain_name,
            chain_size,
        })
    }

    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }

    pub fn inner(&self) -> &IPT {
        &self.inner
    }

    pub fn add_rule<R>(&self, rule: R) -> IPTablesResult<i32>
    where
        R: AsRef<str>,
    {
        self.inner
            .insert_rule(
                &self.chain_name,
                rule.as_ref(),
                self.chain_size.fetch_add(1, Ordering::Relaxed),
            )
            .map(|_| self.chain_size.load(Ordering::Relaxed))
            .inspect_err(|_| {
                self.chain_size.fetch_sub(1, Ordering::Relaxed);
            })
    }

    pub fn remove_rule<R>(&self, rule: R) -> IPTablesResult<()>
    where
        R: AsRef<str>,
    {
        self.inner.remove_rule(&self.chain_name, rule.as_ref())?;

        self.chain_size.fetch_sub(1, Ordering::Relaxed);

        Ok(())
    }
}

impl<IPT> Drop for IPTableChain<IPT>
where
    IPT: IPTables,
{
    fn drop(&mut self) {
        let _ = self.inner.remove_chain(&self.chain_name);
    }
}

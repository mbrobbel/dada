use crate::interpreter::Interpreter;

use super::{invalidated::Invalidated, tenant::Tenant, Permission, PermissionData};

#[derive(Debug)]
pub(crate) struct My {
    /// Owners permissions are invalidated when they are given
    /// away.
    given: Invalidated,

    tenant: Tenant,
}

impl From<My> for PermissionData {
    fn from(m: My) -> Self {
        Self::My(m)
    }
}

impl My {
    pub(super) fn new(_interpreter: &Interpreter<'_>) -> Self {
        Self {
            given: Invalidated::default(),
            tenant: Tenant::default(),
        }
    }

    pub(super) fn give(&self, interpreter: &Interpreter<'_>) -> eyre::Result<Permission> {
        self.check_owned(interpreter)?;
        let permission = Permission::my(interpreter);
        Ok(permission)
    }

    pub(super) fn lease(&self, interpreter: &Interpreter<'_>) -> eyre::Result<Permission> {
        self.given.check_still_valid(interpreter)?;
        Ok(self.tenant.lease(interpreter))
    }

    pub(super) fn give_share(&self, interpreter: &Interpreter<'_>) -> eyre::Result<Permission> {
        self.check_owned(interpreter)?;
        let permission = Permission::our(interpreter);
        Ok(permission)
    }

    pub(super) fn check_read(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        self.given.check_still_valid(interpreter)?;
        self.tenant.cancel_tenant_if_exclusive(interpreter);
        Ok(())
    }

    pub(super) fn check_write(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        self.given.check_still_valid(interpreter)?;
        self.tenant.cancel_tenant(interpreter);
        Ok(())
    }

    pub(crate) fn check_await(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        self.check_owned(interpreter)
    }

    /// Check that giving ownership of this is ok (and do it).
    fn check_owned(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        self.given.invalidate(interpreter)?;
        self.tenant.cancel_tenant(interpreter);
        Ok(())
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.given.is_valid()
    }

    pub(crate) fn peek_tenant(&self) -> Option<Permission> {
        self.tenant.peek_tenant()
    }
}

### Locked Accounts are Inactive

The specification does not dictate any behavior around locked accounts. The assumption is
that a locked account is automatically deactivated, which means any transaction that
refers to such an account throws an error. An account is reactivated when its pending
dispute is resolved.

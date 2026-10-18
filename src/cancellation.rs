//! Periodic, cheap cancellation checks for the long-running operations of this crate.
//!
//! An SBML operation walks a whole document, which can be arbitrarily large, so it has to notice a
//! request to stop instead of running to completion. The request is expressed through the
//! thread-local trigger stack of [`cancel_this`], which is what `read`/`write` of the underlying
//! XML layer already honour; [`CancellationCheck`] is the single place where this crate consults
//! that stack, so every walk can share the same policy.
//!
//! Consulting the trigger stack on every node is measurable (it is a thread-local borrow and a
//! dynamic call), so a walk checks it once every
//! [`CHECK_INTERVAL`](crate::cancellation::CHECK_INTERVAL) steps instead. The *first*
//! step always checks: a walk started under a trigger that has already fired returns immediately
//! rather than after the first interval, even for a tiny document. That is the rule of the XML
//! layer as well — "a trigger that has already fired aborts the walk immediately" — and it is what
//! makes a cancelled operation cheap to abandon.
//!
//! ## Relationship to the XML layer
//!
//! This type is a crate-local mirror of the identical, crate-private
//! `CancellationCheck` of `biodivine-lib-xml-dom` (its `src/cancellation.rs`): the XML layer
//! cannot export the type, and the two crates are separate repositories, so the policy is written
//! out once on each side. The two copies must stay in step — same interval, same first-step rule,
//! same acceptance of an already-fired trigger — because a caller that cancels an SBML operation
//! observes the two crates as one operation: [`Sbml::read_str`](crate::Sbml::read_str) parses
//! through the XML layer's check and then validates through this one. The natural end state is a
//! single implementation that the XML layer owns and this crate re-exports, so that the policy
//! cannot drift; until then, a change to one of the two interval constants has to be mirrored by
//! hand, and the tests of both modules pin the behaviour they share.
//!
//! ## What is checked, and how often
//!
//! A walk hands a `&mut CancellationCheck` down explicitly and calls [`CancellationCheck::tick`]
//! once per visited object; there is no thread-local state of this crate's own, so a nested walk
//! cannot accidentally reset or share the countdown. The phases of
//! [`Sbml::validate`](crate::Sbml::validate) and their bounds are documented on that method.

use crate::SbmlResult;

/// How many steps of a walk pass between two consultations of the cancellation triggers.
pub(crate) const CHECK_INTERVAL: u32 = 1024;

/// A countdown that consults the cancellation triggers of the current thread every fixed number of
/// calls of [`CancellationCheck::tick`].
///
/// A walk creates one check for the whole operation and passes it down, so the countdown is shared
/// by every phase of that operation instead of being restarted by each nested walk. This is the type
/// a package's validation check receives, so that the package's own walk is part of the same
/// cancellable operation as the checks of this crate; see
/// [`ExtensionValidation`](crate::ExtensionValidation) for the contract.
pub struct CancellationCheck {
    /// `false` for operations that must not be cancelled (see
    /// [`CancellationCheck::disabled`]).
    enabled: bool,
    /// Steps left until the next consultation.
    remaining: u32,
}

impl CancellationCheck {
    /// A check that consults the triggers on the first step and then periodically.
    pub(crate) fn new() -> Self {
        Self {
            enabled: true,
            remaining: 0,
        }
    }

    /// A check that never fires, for a walk that cannot report an error.
    ///
    /// A walk that hands its result to a caller which cannot return [`crate::SbmlError`] has no way
    /// to turn a fired trigger into a report, and returning a half-checked document is not an
    /// option, so such a walk keeps its input intact and ignores the triggers instead. Every
    /// cancellable operation of this crate does return an error, so this constructor is currently
    /// exercised only by the tests of this module; it exists to keep the policy — and its two
    /// implementations — complete, rather than to leave the "must not cancel" case to be
    /// reinvented at the first caller that needs it.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "exercised by the unit tests of this module")
    )]
    pub(crate) fn disabled() -> Self {
        Self {
            enabled: false,
            remaining: 0,
        }
    }

    /// Accounts for one step of the walk.
    ///
    /// The returned error names the trigger that fired, so the report of a cancelled operation
    /// tells the caller *why* it stopped.
    ///
    /// This is the one operation of the type: a check is created by the library (by the operation
    /// that is being cancelled) and handed to the code that walks the document, which accounts for
    /// its own steps with this method. A package's check may call it once per element it visits, or
    /// not at all when its own work is bounded - the countdown of the surrounding operation is
    /// unaffected either way.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if this step consults the
    /// triggers and one of them has fired.
    #[inline]
    pub fn tick(&mut self) -> SbmlResult<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.remaining == 0 {
            self.remaining = CHECK_INTERVAL;
            cancel_this::is_cancelled!()?;
        } else {
            self.remaining -= 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SbmlError;

    /// A fired trigger aborts the first step; the steps in between are not consulted; the next
    /// interval checks again. A disabled check never fires.
    #[test]
    fn the_first_step_and_every_interval_consult_the_triggers() {
        let trigger = cancel_this::CancelAtomic::new();
        trigger.cancel();
        let result: SbmlResult<()> = cancel_this::on_atomic(trigger, || {
            let mut check = CancellationCheck::new();
            assert!(
                matches!(check.tick(), Err(SbmlError::Cancelled(_))),
                "the first step checks"
            );
            for _ in 0..CHECK_INTERVAL {
                assert!(check.tick().is_ok(), "the steps in between do not");
            }
            assert!(
                matches!(check.tick(), Err(SbmlError::Cancelled(_))),
                "the next interval checks again"
            );

            let mut disabled = CancellationCheck::disabled();
            assert!(disabled.tick().is_ok());
            Ok(())
        });
        assert!(result.is_ok());
    }

    /// Without a fired trigger the same countdown passes every step, so a walk is only aborted on
    /// request.
    #[test]
    fn every_step_passes_when_no_trigger_has_fired() {
        let mut check = CancellationCheck::new();
        for _ in 0..(CHECK_INTERVAL * 2 + 1) {
            assert!(check.tick().is_ok());
        }
        let mut disabled = CancellationCheck::disabled();
        for _ in 0..(CHECK_INTERVAL * 2 + 1) {
            assert!(disabled.tick().is_ok());
        }
    }
}

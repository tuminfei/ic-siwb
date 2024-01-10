/*!
`ic_siwe` is the Rust library that facilitates the integration of Ethereum wallet-based authentication with applications on
the Internet Computer (IC) platform. The library provides all necessary tools for integrating Sign-In with
Ethereum (SIWE) into IC canisters, from generating SIWE messages to creating delegate identities.

`ic_siwe` is part of the [ic-siwe](https://github.com/kristoferlund/ic-siwe) project that enables Ethereum wallet-based
authentication for applications on the Internet Computer (IC) platform. The goal of the project is to enhance the interoperability
between Ethereum and the Internet Computer platform, enabling developers to build applications that leverage the strengths of both platforms.

## Key Features
- **Ethereum Wallet Sign-In**: Enables Ethereum wallet sign-in for IC applications. Sign in with any eth wallet to generate an
IC identity and session.
- **Session Identity Uniqueness**: Ensures that session identities are specific to each application's context, preventing cross-app
identity misuse.
- **Consistent Principal Generation**: Guarantees that logging in with an Ethereum wallet consistently produces the same Principal,
irrespective of the client used.
- **Direct Ethereum Address to Principal Mapping**: Creates a one-to-one correlation between Ethereum addresses and Principals
within the scope of the current application.
- **Timebound Sessions**: Allows developers to set expiration times for sessions, enhancing security and control.

## Prebuilt `ic_siwe_provider` canister

While the `ic_siwe` library can be used to build custom solutions, the
[ic-siwe-provider](https://github.com/kristoferlund/ic-siwe/tree/main/packages/ic_siwe_provider) canister provides a
prebuilt solution for handling the login flow and delegating identities within the IC platform.

Developers can integrate this canister into their projects with minimal coding effort by adding it to their `dfx.json`.
This approach simplifies the development process, focusing on configuration over coding.

## SIWE Standard

[ERC-4361: Sign-In with Ethereum](https://eips.ethereum.org/EIPS/eip-4361) - Off-chain authentication for
Ethereum accounts to establish sessions

`ic_siwe` implements most parts of the Sign In with Ethereum (SIW standard,
[EIP-4361](https://eips.ethereum.org/EIPS/eip-4361) with some notable exceptions:

- `nonce` - The SIWE standard requires that each SIWE message has a unique nonce. In the context of this
  implementation, the nonce don't add any additional security to the login flow. If random nonces are
  required, the `nonce` feature flag can be enabled. When this feature is enabled, the nonce is generated
  using a cryptographically secure random number generator.

- `not-before`, `request-id`, `resources` - Not implemented. These fields are marked as OPTIONAL in the
  SIWE standard and are not required for current implementation.

# Login flow

Three canister methods need to be exposed to implement the login flow: `prepare_login`, `login`, and `get_delegation`.

## `prepare_login`
- The `prepare_login` method is called by the frontend application to initiate the login flow. The method
  takes the user's Ethereum address as a parameter and returns a SIWE message. The frontend application
  uses the SIWE message to prompt the user to sign the message with their Ethereum wallet.
- See: [`login::prepare_login`]

## `login`
- The `login` method is called by the frontend application after the user has signed the SIWE message. The
  method takes the user's Ethereum address, signature, and session identity as parameters. The method
  verifies the signature and Ethereum address and returns a delegation.
- See: [`login::login`]

## `get_delegation`
- The `get_delegation` method is called by the frontend application after a successful login. The method
  takes the delegation expiration time as a parameter and returns a delegation.
- The `get_delegation` method is not mirrored by one function in the `ic_siwe` library. The creation of delegate
  identities requires setting the certified data of the canister. This should not be done by the library, but by the
  implementing canister.
- Creating a delegate identity involves interacting with the following `ic_siwe` functions: [`delegation::generate_seed`],
  [`delegation::create_delegation`], [`delegation::create_delegation_hash`], [`delegation::witness`],
  [`delegation::create_certified_signature`].
- For a full implementation example, see the
  [`ic_siwe_provider`](https://github.com/kristoferlund/ic-siwe/tree/main/packages/ic_siwe_provider) canister.

The login flow is illustrated in the following diagram:

```text

                                ┌────────┐                                        ┌────────┐                              ┌─────────┐
                                │Frontend│                                        │Canister│                              │EthWallet│
   User                         └───┬────┘                                        └───┬────┘                              └────┬────┘
    │      Push login button       ┌┴┐                                                │                                        │
    │ ────────────────────────────>│ │                                                │                                        │
    │                              │ │                                                │                                        │
    │                              │ │          prepare_login(eth_address)           ┌┴┐                                       │
    │                              │ │ ─────────────────────────────────────────────>│ │                                       │
    │                              │ │                                               └┬┘                                       │
    │                              │ │                OK, siwe_message                │                                        │
    │                              │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                        │
    │                              │ │                                                │                                        │
    │                              │ │                                   Sign siwe_message                                    ┌┴┐
    │                              │ │ ──────────────────────────────────────────────────────────────────────────────────────>│ │
    │                              │ │                                                │                                       │ │
    │                              │ │                  Ask user to confirm           │                                       │ │
    │ <───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────│ │
    │                              │ │                                                │                                       │ │
    │                              │ │                          OK                    │                                       │ │
    │  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ >│ │
    │                              │ │                                                │                                       └┬┘
    │                              │ │                                      OK, signature                                      │
    │                              │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
    │                              │ │                                                │                                        │
    │                              │ │────┐                                           │                                        │
    │                              │ │    │ Generate random session_identity          │                                        │
    │                              │ │<───┘                                           │                                        │
    │                              │ │                                                │                                        │
    │                              │ │login(eth_address, signature, session_identity)┌┴┐                                       │
    │                              │ │ ─────────────────────────────────────────────>│ │                                       │
    │                              │ │                                               │ │                                       │
    │                              │ │                                               │ │────┐                                  │
    │                              │ │                                               │ │    │ Verify signature and eth_address │
    │                              │ │                                               │ │<───┘                                  │
    │                              │ │                                               │ │                                       │
    │                              │ │                                               │ │────┐                                  │
    │                              │ │                                               │ │    │ Prepare delegation               │
    │                              │ │                                               │ │<───┘                                  │
    │                              │ │                                               └┬┘                                       │
    │                              │ │     OK, canister_pubkey, delegation_expires    │                                        │
    │                              │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                        │
    │                              │ │                                                │                                        │
    │                              │ │      get_delegation(delegation_expires)       ┌┴┐                                       │
    │                              │ │ ─────────────────────────────────────────────>│ │                                       │
    │                              │ │                                               └┬┘                                       │
    │                              │ │                 OK, delegation                 │                                        │
    │                              │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                        │
    │                              │ │                                                │                                        │
    │                              │ │────┐                                           │                                        │
    │                              │ │    │ Create delegation identity                │                                        │
    │                              │ │<───┘                                           │                                        │
    │                              └┬┘                                                │                                        │
    │ OK, logged in with            │                                                 │                                        │
    │ Principal niuiu-iuhbi...-oiu  │                                                 │                                        │
    │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                                  │                                        │
  User                          ┌───┴────┐                                        ┌───┴────┐                              ┌────┴────┐
                                │Frontend│                                        │Canister│                              │EthWallet│
                                └────────┘                                        └────────┘                              └─────────┘
```

# Crate features

The library has one optional feature that is disabled by default.

* `nonce` - Enables the generation of nonces for SIWE messages. This feature initializes a random number
generator with a seed from the management canister. The random number generator then is used to generate
unique nonces for each generated SIWE message. Nonces don't add any additional security to the SIWE login
flow but are required by the SIWE standard. When this feature is disabled, the nonce is always set to the
hex encoded string `Not in use`.

*/
pub mod delegation;
pub mod eth;
pub(crate) mod hash;
pub(crate) mod init;
pub mod login;
pub(crate) mod rand;
pub mod settings;
pub mod signature_map;
pub mod siwe;
pub(crate) mod time;

pub use init::init;

use settings::Settings;
use siwe::SiweMessage;
use std::{cell::RefCell, collections::HashMap};

#[cfg(feature = "nonce")]
use rand_chacha::ChaCha20Rng;

thread_local! {
    // The random number generator is used to generate nonces for SIWE messages. This feature is
    // optional and can be enabled by setting the `nonce` feature flag.
    #[cfg(feature = "nonce")]
    static RNG: RefCell<Option<ChaCha20Rng>> = RefCell::new(None);

    // The settings control the behavior of the SIWE library. The settings must be initialized
    // before any other library functions are called.
    static SETTINGS: RefCell<Option<Settings>> = RefCell::new(None);

    // SIWE messages are stored in global state during the login process. The key is the
    // Ethereum address as a byte array and the value is the SIWE message. After a successful
    // login, the SIWE message is removed from the state.
    static SIWE_MESSAGES: RefCell<HashMap<Vec<u8>, SiweMessage>> = RefCell::new(HashMap::new());
}

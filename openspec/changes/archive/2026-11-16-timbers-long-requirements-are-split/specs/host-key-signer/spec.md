## MODIFIED Requirements

### Requirement: The given host key is made on first use

A host key bound as the given one, at `timber/host/signing.key` in the
user's data directory (`$XDG_DATA_HOME`, or `~/.local/share` where that
variable is unset or empty), SHALL, on a sign that finds no key file
there, make its directory with mode 0700 where it makes it, make an
Ed25519 key in OpenSSH's format with no passphrase, mode 0600, write its
public half beside it as `signing.key.pub`, one line naming the signer,
and sign with it (118, C.8).

#### Scenario: The first sign

- **WHEN** the given host key is asked to sign a claim as the host's name and no file is at its path
- **THEN** `timber/host` is made under the data directory with mode 0700, the key and `signing.key.pub` are made in it, the key's mode is 0600, the envelope verifies against the public half `signing.key.pub` holds, and that line, read as a policy listing, tells the envelope as made by the host's name

#### Scenario: The second sign

- **WHEN** the given host key signs a second claim
- **THEN** it signs with the key the first sign made, and neither file is written again

#### Scenario: A key others may read at the given path

- **WHEN** a key file of mode 0644 is at the given path
- **THEN** sign is refused naming the file, its mode and `chmod 600`, and the file is not replaced

#### Scenario: Two first signs at once

- **WHEN** two processes sign with the given host key at once and no key is there
- **THEN** one key is made, and both envelopes verify against it

#### Scenario: Verify makes nothing

- **WHEN** the host key bound verify-only verifies an envelope on a host with no key at the given path
- **THEN** no file is made

#### Scenario: A directory that cannot be written

- **WHEN** the given key's directory cannot be made or written
- **THEN** sign answers unreadable naming the path, carrying the operating system's words, and no envelope is answered

## ADDED Requirements

### Requirement: Only a sign makes the given host key, and none is replaced

Only a sign SHALL make the given host key. A key file that is there
SHALL be read as any bound key is, never replaced (118, C.8, core.54).

#### Scenario: A key already at the given path

- **WHEN** a key file of mode 0600 is at the given path and the given host key is asked to sign
- **THEN** it signs with that key, and neither the key nor `signing.key.pub` is written

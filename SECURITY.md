# Security Policy

## Supported versions

Aipo is under active development. Security fixes are applied to the latest code on the `main` branch and, when releases are published, to the latest supported release line when practical.

Older development snapshots and superseded releases should not be assumed to receive security fixes.

## Reporting a vulnerability

Please **do not** open a public issue for an undisclosed security vulnerability.

Use GitHub's private vulnerability reporting flow from the repository's **Security** tab when it is available. Include:

- affected component and version/commit;
- a clear description of the issue and its security impact;
- minimal reproduction steps or proof of concept;
- any known prerequisites or exploit constraints;
- suggested remediation, if known.

If private vulnerability reporting is temporarily unavailable, contact a repository maintainer through an existing private channel and request a private security discussion before sharing details.

We will acknowledge reports as soon as practical, validate the issue, coordinate a fix and disclosure timeline, and credit reporters who want attribution.

## Scope

Security-sensitive areas include, among others:

- parser, compiler and bytecode validation;
- virtual machine and runtime isolation;
- JavaScript backend parity when differences can change security-relevant behavior;
- CLI input handling and filesystem/process boundaries;
- dependency and build-chain integrity;
- CI/CD workflows and release artifacts.

## Safe testing

Do not perform testing that harms third parties, destroys data, disrupts services, or exposes credentials or private information. Keep reproduction data synthetic and minimal whenever possible.

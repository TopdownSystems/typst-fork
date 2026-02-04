# About This Fork

This repository is a fork of [Typst](https://github.com/typst/typst), the modern typesetting system.

## Purpose

This fork exists for one specific purpose: **to add the `wrap` and `masthead` elements for text-flow layouts**.

These features enable magazine-style text flow where content wraps around images, sidebars, and other elements - functionality commonly needed in professional document production.

## What This Fork Is

- A specialized extension of Typst with text-flow features (`wrap` and `masthead`)
- Actively maintained to stay in sync with the upstream Typst repository
- Used internally by TopdownSystems for production document generation

## What This Fork Is NOT

- **Not a replacement for Typst** - We encourage everyone to use the official [Typst](https://github.com/typst/typst) project
- **Not a competing project** - We fully support the Typst team and their vision
- **Not a permanent divergence** - We hope these features may eventually be considered for upstream inclusion

## Staying in Sync

We make strong efforts to keep this fork synchronized with the main Typst repository:

- **Automated daily sync** via GitHub Actions checks for upstream changes
- **Main branch** mirrors `typst/typst:main` as closely as possible
- **Feature branch** (`feature/text-flow`) contains our additions on top of the latest upstream code

## Documentation

- [Text-Flow User Guide](docs/TDSC-Doc%20and%20Examples/TEXT-FLOW-GUIDE.md) - How to use `wrap` and `masthead`
- [Testing Guide](docs/TDSC-Doc%20and%20Examples/TESTING-GUIDE.md) - For developers testing the feature
- [Examples](docs/TDSC-Doc%20and%20Examples/examples/) - Sample documents and compiled PDFs

## License

This fork maintains the same [Apache 2.0 License](LICENSE) as the original Typst project.

## Acknowledgments

All credit for Typst goes to the [Typst team](https://github.com/typst/typst/graphs/contributors). This fork builds upon their excellent work.

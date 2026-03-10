# Introduction

**cohdl** is a text-based hardware description language for designing printed circuit boards (PCBs). Instead of drawing schematics in a graphical editor, you describe your circuits in `.cohdl` source files — plain text that is easy to read, review, and track with version control.

## Why cohdl?

Traditional PCB design tools store schematics in opaque binary or XML formats that are difficult to diff, merge, and review in a typical software workflow. cohdl takes a different approach:

- **Text-first workflow.** Designs live in `.cohdl` source files that can be edited in any text editor. Diffs are meaningful and code review works just like it does for software.
- **Version-control friendly.** Because the source format is plain text, Git (or any VCS) handles branching, merging, and history naturally. No more conflicting binary blobs.
- **Language server support.** cohdl ships with a built-in LSP server, giving you real-time diagnostics, go-to-definition, and completions in editors that support the Language Server Protocol.
- **KiCad backend.** cohdl compiles designs to KiCad project files, so you can hand off to KiCad for layout, manufacturing outputs, and the broader KiCad ecosystem.
- **Design rule checking.** A dedicated DRC pass catches electrical and structural errors before you ever open a layout tool.

## How it works

A cohdl project is a collection of `.cohdl` files that describe parts, devices, nets, and modules. The compiler parses these files, runs semantic analysis and design rule checks, and then emits output through a backend (currently KiCad). The result is a set of files you can open directly in the target EDA tool.

## What's in this book

This reference covers everything you need to go from installation to a complete design:

- **Getting Started** walks you through installing the toolchain and building your first project.
- **Language Reference** details every construct in the language — traits, devices, parts, types, functions, modules, packages, nets, designators, and DRC rules.
- **Backends** documents the available output formats, starting with KiCad.
- **CLI Reference** describes the command-line interface and its options.

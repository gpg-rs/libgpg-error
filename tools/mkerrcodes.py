#!/usr/bin/env python3
# Script to generate constant wrappers for error codes
# from upstream sources.
from pathlib import Path

root = Path(__file__).resolve().parent.parent


def read_codes(path, list):
    with open(root / path, encoding='utf-8') as f:
        for l in f:
            parts = l.split()
            if len(parts) < 2:
                continue
            try:
                int(parts[0])
            except ValueError:
                continue
            list.append((parts[1], parts[0]))


sources = []
codes = []
errnos = []

for (f, l) in [('err-sources.h.in', sources), ('err-codes.h.in', codes), ('errnos.in', errnos)]:
    read_codes(Path('vendor') / f, l)

with open(root / 'libgpg-error-sys/src/consts.rs', 'w', encoding='utf-8', newline='\n') as out:
    for (name, val) in sources:
        out.write(f"pub const {name}: gpg_err_source_t = {val};\n")
    for (name, val) in codes:
        out.write(f"pub const {name}: gpg_err_code_t = {val};\n")
    for (name, val) in errnos:
        out.write(
            f"pub const GPG_ERR_{name}: gpg_err_code_t = GPG_ERR_SYSTEM_ERROR | {val};\n")
with open(root / 'src/consts.rs', 'w', encoding='utf-8', newline='\n') as out:
    out.write('impl Error{\n')
    for (name, _) in sources:
        out.write(
            f"pub const {name.removeprefix('GPG_ERR_')}: ErrorSource = ffi::{name};\n")
    for (name, _) in codes:
        out.write(
            f"pub const {name.removeprefix('GPG_ERR_')}: Self = Self(ffi::{name});\n")
    for (name, _) in errnos:
        out.write(f"pub const {name}: Self = Self(ffi::GPG_ERR_{name});\n")
    out.write('}\n')

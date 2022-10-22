.PHONY: gen
gen: src/consts.rs libgpg-error-sys/src/consts.rs

src/consts.rs libgpg-error-sys/src/consts.rs: vendor/err-sources.h.in vendor/err-codes.h.in vendor/errnos.in
	./tools/mkerrcodes.py

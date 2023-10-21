.POSIX:

src/: a53.svd
	@# Generate lib.rs from the SVD
	svd2rust --target none -i $?
	@# Generate a new src/ from the auto-generated lib.rs
	rm -rf src/
	form -i lib.rs -o src/
	rm lib.rs
	@# lib.rs and the form output are unformatted - clean them up
	cargo fmt

clean:
	rm -rf src/

.PHONY: clean

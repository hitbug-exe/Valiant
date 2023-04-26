{ pkgs }: {
	deps = [
		pkgs.toybox
  pkgs.rustc
		pkgs.rustfmt
		pkgs.cargo
		pkgs.cargo-edit
    pkgs.rust-analyzer
	];
}
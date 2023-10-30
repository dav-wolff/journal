{
	description = "journal";
	
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
	};
	
	outputs = { self, nixpkgs }:
		let
			pkgs = nixpkgs.legacyPackages.x86_64-linux;
		in let
			journal = let
					manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
				in pkgs.rustPlatform.buildRustPackage {
					pname = manifest.name;
					version = manifest.version;
					
					cargoLock.lockFile = ./Cargo.lock;
					src = pkgs.lib.cleanSource ./.;
				};
		in {
			packages.x86_64-linux = {
				inherit journal;
				
				default = journal;
			};
			
			devShells.x86_64-linux = {
				default = pkgs.mkShell {
					inputsFrom = [
						journal
					];
					
					buildInputs = with pkgs; [
						rust-analyzer
						clippy
					];
				};
			};
		};
}

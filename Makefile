
push-docs:
	cargo doc --lib -p loggerv
	echo "<meta http-equiv=refresh content=0;url=loggerv/index.html>" > target/doc/index.html
	ghp-import -n target/doc
	git push -qf "git@github.com:clux/loggerv.git" gh-pages

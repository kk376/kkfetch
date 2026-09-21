srpm:
	mkdir -p $(outdir)
	VERSION=$$(rpm -q --qf "%{VERSION}\n" --specfile $(spec) | head -n 1) ; \
	curl -sL "https://github.com/kk376/kkfetch/archive/refs/tags/v$${VERSION}.tar.gz" -o "$$(dirname $(spec))/kkfetch-$${VERSION}.tar.gz" && \
	rpmbuild -bs --define "_sourcedir $$(dirname $(spec))" --define "_srcrpmdir $(outdir)" $(spec)

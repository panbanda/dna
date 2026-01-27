FROM scratch

COPY dna /usr/local/bin/dna

ENTRYPOINT ["/usr/local/bin/dna"]

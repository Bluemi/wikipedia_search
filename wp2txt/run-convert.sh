#!/bin/bash

docker run --rm -v $PWD/../data/input:/data -w /data yohasebe/wp2txt wp2txt -i raw/dewiki-20241220-pages-articles-multistream.xml.bz2 --summary-only -o ./summary

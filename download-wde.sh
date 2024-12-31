#!/bin/bash

# download all articles
# wget -O "data/dewiki-latest-pages-articles.xml.bz2" "https://dumps.wikimedia.org/dewiki/latest/dewiki-latest-pages-articles.xml.bz2"

# download some artices
wget -O "data/dewiki-latest-pages-articles-multistream1.xml-p1p297012.bz2" "https://dumps.wikimedia.org/dewiki/latest/dewiki-latest-pages-articles-multistream1.xml-p1p297012.bz2"
# unpack
bzip2 -dk "data/dewiki-latest-pages-articles-multistream1.xml-p1p297012.bz2"

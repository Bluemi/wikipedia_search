#!/bin/bash

case "$1" in
	m)
		python3 src/model_encoding.py
		;;
	d)
		python3 src/wiki_data.py
		;;
esac

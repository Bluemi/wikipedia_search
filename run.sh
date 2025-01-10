#!/bin/bash

case "$1" in
	r)
		shift
		python3 src/main.py "$@"
		;;
	m)
		python3 src/model_encoding.py
		;;
	d)
		python3 src/wiki_data.py
		;;
	*)
		echo "use m or d"
		;;
esac

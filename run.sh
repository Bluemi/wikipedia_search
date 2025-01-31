#!/bin/bash

case "$1" in
	r)
		shift
		python3 src/search_graph.py "$@"
		;;
	e)
		shift
		python3 src/encode_text.py "$@"
		;;
	c)
		shift
		python3 src/create_graph.py "$@"
		;;
	m)
		# python3 src/model_tests/model_encoding.py
		python3 src/model_tests/jina.py
		;;
	d)
		python3 src/wiki_data.py
		;;
	*)
		echo "use m or d"
		;;
esac


(echo -n '{"title": "Writing", "content": "'; base64 hello_world.tar.gz; echo '"}') |
curl -H 'Content-Type: application/json' \
	-d @- localhost:3000/api/create

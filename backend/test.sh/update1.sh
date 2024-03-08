# Editing content
(echo -n '{"title": "Writing hello world in rust!", "content": "'; base64 hello_world_in_rust_fixed.tar.gz; echo '"}') |

curl -X PUT -H 'Content-Type: application/json' \
	-d @- localhost:3000/api/update

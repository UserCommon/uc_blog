# curl -X PUT -v localhost:3000/api/update \
# 	--header 'Content-Type: application/json' \
# 	--data '{"title": "Hey, thats it!", "content": "#Start", "id": 7}'


# (echo -n '{"title": "Writing hello world in rust!", "content": "'; base64 hello_world_fixed.tar.gz; echo '"}') |
# curl -H 'Content-Type: application/json' \
# 	-d @- localhost:3000/api/create

# Editing title
(echo -n '{"title": "Writing hello world in rust!", "new_title": "Writing_hello world1"}') |
curl -X PUT -H 'Content-Type: application/json' \
	-d @- localhost:3000/api/update

# Editiing Content

# Editing Content AND Title


# (echo -n '{"title": "Writing", "content": "'; base64 hello_world.tar.gz; echo '"}') |
# curl -H 'Content-Type: application/json' \
# 	-d @- localhost:3000/api/create

curl -X POST -F title="Hey, It's working!" -F archive=@hello_world.tar.gz localhost:3000/api/create

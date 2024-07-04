
# (echo -n '{"title": "Writing", "content": "'; base64 hello_world.tar.gz; echo '"}') |
# curl -H 'Content-Type: application/json' \
# 	-d @- localhost:3000/api/create

curl -u usercommon:wow -X POST -F title="creating" -F archive=@hello_world.tar.gz localhost:3001/api/v1/articles

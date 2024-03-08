# Editing content
curl -X PUT -F title="Hey, It's working1!" -F new_title="zxc" -F archive=@hello_world.tar.gz localhost:3000/api/update

UPDATE opsml_deck_registry SET 
app_env = $1, 
name = $2,
space = $3,
major = $4, 
minor = $5,
patch = $6, 
version = $7, 
cards = $8,
username = $9,
opsml_version = $10
WHERE uid = $11;
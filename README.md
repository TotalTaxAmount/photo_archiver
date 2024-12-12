# Archive and do cool stuff with google phots


#### TODO:
- [ ] Write the frontend
  - [x] Login page
  - [ ] Homepage
  - [ ] User settings
- [ ] Fix the build in `flake.nix`
  - [x] Build backend and frontend
  - [ ] Config stuff the nix way
- [ ] Google APIs
  - [x] Authenticate
  - [x] Get user info
  - [ ] Photos
    - [x] List them
    - [ ] Download 
- [ ] Misc
  - [ ] Use more type alias: (ex: Arc<Mutex<**Whatever**>> -> Shared**Whatever**)
  - [ ] Give users a role (Admin, Member, etc)
  - [ ] Write documentation
  - [ ] Migration stuff
  - [ ] Refresh tokens (Use the ones from google, maybe have a way to refresh photo archiver JWT tokens?)
  - [ ] Write tests
  - [ ] Endpoints like /api/users/userinfo should have a id parameter so higher privilege users can get lower privilege user info

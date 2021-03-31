allow(actor: AuthenticatedUser, action, resource) if actor.is_admin();
allow(actor: AuthenticatedUser, action, resource) if resource.starts_with("/judging") and actor.is_judge();
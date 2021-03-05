allow(actor: AuthenticatedUser, "GET", resource) if actor.is_admin();

allow(actor: AuthenticatedUser, "GET", "/public/auth/protected") if actor.is_commentator();

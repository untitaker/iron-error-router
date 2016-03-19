# iron-error-router [![Build status](https://travis-ci.org/untitaker/iron-error-router.svg?branch=master)](https://travis-ci.org/untitaker/iron-error-router)

A Iron middleware for showing custom error pages for specific statuses.

Currently the middleware will check if a body has not yet been set for the
response, and will invoke a custom `AfterMiddleware` or `Handler`,
depending on the response's status.

See `/examples/` for usage.

## License

Licensed under the MIT, see `LICENSE`.

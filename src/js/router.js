// Simple hash-based router
class Router {
  constructor() {
    this.routes = {};
    this.currentRoute = null;
    window.addEventListener('hashchange', () => this.handleRoute());
  }

  register(path, handler) {
    this.routes[path] = handler;
  }

  navigate(path) {
    window.location.hash = path;
  }

  handleRoute() {
    const hash = window.location.hash.slice(1) || '/';
    const [path, ...params] = hash.split('/').filter(Boolean);
    const fullPath = '/' + path;

    // Check for exact match first
    if (this.routes[fullPath]) {
      this.currentRoute = fullPath;
      this.routes[fullPath](params);
      return;
    }

    // Check for parameterized routes
    for (const route of Object.keys(this.routes)) {
      const routeParts = route.split('/').filter(Boolean);
      const pathParts = hash.split('/').filter(Boolean);

      if (routeParts.length !== pathParts.length) continue;

      const params = {};
      let match = true;

      for (let i = 0; i < routeParts.length; i++) {
        if (routeParts[i].startsWith(':')) {
          params[routeParts[i].slice(1)] = pathParts[i];
        } else if (routeParts[i] !== pathParts[i]) {
          match = false;
          break;
        }
      }

      if (match) {
        this.currentRoute = route;
        this.routes[route](params);
        return;
      }
    }

    // Default to projects
    this.navigate('/projects');
  }

  start() {
    this.handleRoute();
  }
}

export const router = new Router();

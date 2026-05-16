# Roadmap — Landing Page

> **Nota:** Este roadmap ya no aplica al proyecto actual.
> El proyecto de Monitoreo de Infraestructura Regional (ADR 0020) no requiere landing page tradicional.
> En su lugar, el acceso es a través del dashboard de monitoreo con autenticación.

---

## Estado: NO APLICA

El sistema de monitoreo de infraestructura es una herramienta interna institucional que:

* Requiere autenticación (RBAC)
* No es un producto público
* No captura leads
* No tiene смысл un landing page comercial

---

## En su lugar

El acceso al sistema es a través de:

```
/login → autenticación PASETO
/dashboard → KPIs de monitoreo
```

---

## Si en el futuro se necesita un portal público

Se podría crear una página de acceso institucional con:

* Información sobre el sistema de monitoreo
* Enlaces a документация
* Contacto para soporte técnico
* Estado del sistema (uptime)

Pero esto no es una prioridad para el MVP.

---

**Referencia:** ADR 0020 (Monitoreo de Infraestructura Regional)
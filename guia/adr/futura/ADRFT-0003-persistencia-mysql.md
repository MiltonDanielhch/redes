# ADR (futuro) — Persistencia con MySQL

| Campo               | Valor                                                                   |
| ------------------- | ----------------------------------------------------------------------- |
| **Estado**          | ⏳ Pendiente                                                            |
| **Fecha**           | 2026                                                                    |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                        |
| **Relacionado con** | ADR 0004 (PostgreSQL), ADR 0013 (Infraestructura), ADR 0008 (Auth)    |

---

# Contexto

MySQL es una alternativa a PostgreSQL con ventajas en algunos escenarios:

* Mayor disponibilidad de hosting (ej: PlanetScale, Cloudflare D1, Railway)
* Sintaxis más familiar para equipos con experiencia en MySQL
* Mejor rendimiento en cargas lectura/escritura simples
* Ecosystem más grande en herramientas de terceros

---

# Motivación

Considerar MySQL si:

1. **Unificación con otros proyectos** - Si la mayoría de tus proyectos usan MySQL, mantener una sola base reduce complejidad operacional
2. **Hosteo específico** - Algunos servicios de hosting ofrecen MySQL gratuito pero no PostgreSQL
3. **Equipo con experiencia** - Si el equipo tiene más experiencia con MySQL, reduces curva de aprendizaje

---

# Cambios requeridos en otros ADRs

Para migrar de PostgreSQL a MySQL, actualizar:

| ADR | Cambios |
|-----|---------|
| ADR 0004 | Cambiar título a "Persistencia con MySQL", actualizar decisiones de migrate SQLx |
| ADR 0005 | Actualizar sintaxis de migraciones (diferencias en tipos: `SERIAL` → `AUTO_INCREMENT`, `TEXT` → `VARCHAR`) |
| ADR 0006 | Actualizar schema de RBAC y auditoría |
| ADR 0008 | Revisar Auth tokens y refresh tokens |
| ADR 0010 | Actualizar testing de integración |
| ADR 0013 | Cambiar servicio de `postgres` a `mysql` en Docker Compose |
| ADR 0019 | Cambiar de PostgreSQL a MySQL |

---

# Decisiones técnicas

## Motor recomendado

* **MySQL 8.0** o **MariaDB 10.11** (más maduro, licencia open source completa)

## Diferencias clave con PostgreSQL

| Aspecto | PostgreSQL | MySQL |
|---------|-----------|-------|
| JSON | Nativo, consultas avanzadas | JSON solo desde MySQL 5.7 |
| Tipos compostos | Soportado | No soportado |
| Full-text search | Nativo | Necesita configuración |
| Window functions | Nativo | Desde MySQL 8.0 |
| CTEs | Nativo | Desde MySQL 8.0 |
| Concurrencia | MVCC nativo | InnoDB con auto-commit |

## Schema esperado

```sql
-- MariaDB/MySQL
CREATE TABLE users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

---

# Backups

MySQL tiene más opciones de backups:

* `mysqldump` - básico
* **XtraBackup** - hot backup, recomendado para producción
* **MariaDB Backup** - si usa MariaDB
*-cloud snapshots via Coolify

---

# Stack Compatible

| Componente | Estado MySQL |
|------------|--------------|
| Axum + SQLx | ✅ sqlx con `mysql` feature |
| Apalis jobs | ✅ funciona con MySQL |
| Litestream | ❌ NO funciona (solo SQLite) |
| Coolify | ✅ MySQL disponible en templates |

---

# Desventajas de MySQL vs PostgreSQL para este proyecto

1. **JSON limitado** - El monitoreo requiere almacenar métricas como JSON
2. **Sin tipos compuestos** - Para estructuras de red complejas
3. **Litestream no funciona** - Backup/replicación diferente
4. **Menor adopción en comunidad Rust** - Menos ejemplos y soporte

---

# Recomendación

**Mantener PostgreSQL** como elección default. MySQL solo si:

* Tenés otros proyectos que usan MySQL y querés unificar
* El hosting específico no ofrece PostgreSQL
* El equipo tiene experiencia limitada con PostgreSQL
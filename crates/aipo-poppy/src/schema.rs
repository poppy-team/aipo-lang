//! Aipo Host Schema (AHS) definition for the Poppy Game Engine profile.

use aipo_host::ahs::{
    FieldSchema, FunctionSchema, HandleSchema, HostSchema, ModuleSchema, ParamSchema, TypeRef,
    TypeSchema,
};

/// Builds the canonical [`HostSchema`] representing the Poppy game profile.
///
/// This schema describes modules, types, handles, functions, and capabilities as data,
/// so compiler, LSP, and tooling can discover the surface without linking to an engine.
#[must_use]
pub fn poppy_schema() -> HostSchema {
    HostSchema {
        host: "poppy".to_string(),
        version: "0.1.0".to_string(),
        modules: vec![ModuleSchema {
            name: "poppy".to_string(),
            docs: Some("Poppy Game Engine reference profile for Aipo".to_string()),
            capabilities: vec!["poppy".to_string()],
            types: vec![
                TypeSchema {
                    name: "Vec2".to_string(),
                    docs: Some("2D vector with x and y components".to_string()),
                    fields: vec![
                        FieldSchema {
                            name: "x".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                        },
                        FieldSchema {
                            name: "y".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                        },
                    ],
                },
                TypeSchema {
                    name: "Transform".to_string(),
                    docs: Some("Entity position in 2D space".to_string()),
                    fields: vec![
                        FieldSchema {
                            name: "x".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                        },
                        FieldSchema {
                            name: "y".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                        },
                    ],
                },
            ],
            handles: vec![HandleSchema {
                name: "Entity".to_string(),
                docs: Some("Generational handle to a game entity".to_string()),
                operations: vec![
                    "position".to_string(),
                    "set_position".to_string(),
                    "velocity".to_string(),
                    "set_velocity".to_string(),
                    "tag".to_string(),
                ],
                capabilities: vec!["poppy.ecs".to_string()],
                stale_is_none: false,
            }],
            values: Vec::new(),
            functions: vec![
                FunctionSchema {
                    name: "spawn".to_string(),
                    docs: Some("Spawns a new entity with a tag and initial position".to_string()),
                    params: vec![
                        ParamSchema {
                            name: "tag".to_string(),
                            ty: TypeRef {
                                name: "String".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                        ParamSchema {
                            name: "x".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                        ParamSchema {
                            name: "y".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                    ],
                    returns: Some(TypeRef {
                        name: "Entity".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "despawn".to_string(),
                    docs: Some("Queues an entity for removal at the next safe point".to_string()),
                    params: vec![ParamSchema {
                        name: "entity".to_string(),
                        ty: TypeRef {
                            name: "Entity".to_string(),
                            nullable: false,
                            args: Vec::new(),
                        },
                        is_mut: false,
                        is_optional: false,
                        docs: None,
                    }],
                    returns: None,
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "query".to_string(),
                    docs: Some("Queries all live entities with a given tag".to_string()),
                    params: vec![ParamSchema {
                        name: "tag".to_string(),
                        ty: TypeRef {
                            name: "String".to_string(),
                            nullable: false,
                            args: Vec::new(),
                        },
                        is_mut: false,
                        is_optional: false,
                        docs: None,
                    }],
                    returns: Some(TypeRef {
                        name: "List".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "get_position".to_string(),
                    docs: Some("Gets the current position of an entity".to_string()),
                    params: vec![ParamSchema {
                        name: "entity".to_string(),
                        ty: TypeRef {
                            name: "Entity".to_string(),
                            nullable: false,
                            args: Vec::new(),
                        },
                        is_mut: false,
                        is_optional: false,
                        docs: None,
                    }],
                    returns: Some(TypeRef {
                        name: "Vec2".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "set_position".to_string(),
                    docs: Some("Sets the position of an entity".to_string()),
                    params: vec![
                        ParamSchema {
                            name: "entity".to_string(),
                            ty: TypeRef {
                                name: "Entity".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                        ParamSchema {
                            name: "x".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                        ParamSchema {
                            name: "y".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                    ],
                    returns: None,
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "get_velocity".to_string(),
                    docs: Some("Gets the current velocity of an entity".to_string()),
                    params: vec![ParamSchema {
                        name: "entity".to_string(),
                        ty: TypeRef {
                            name: "Entity".to_string(),
                            nullable: false,
                            args: Vec::new(),
                        },
                        is_mut: false,
                        is_optional: false,
                        docs: None,
                    }],
                    returns: Some(TypeRef {
                        name: "Vec2".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "set_velocity".to_string(),
                    docs: Some("Sets the velocity of an entity".to_string()),
                    params: vec![
                        ParamSchema {
                            name: "entity".to_string(),
                            ty: TypeRef {
                                name: "Entity".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                        ParamSchema {
                            name: "vx".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                        ParamSchema {
                            name: "vy".to_string(),
                            ty: TypeRef {
                                name: "Float".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                    ],
                    returns: None,
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "random_float".to_string(),
                    docs: Some("Generates a pseudo-random float in [0.0, 1.0)".to_string()),
                    params: Vec::new(),
                    returns: Some(TypeRef {
                        name: "Float".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.random".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "random_int".to_string(),
                    docs: Some("Generates a pseudo-random integer in [min, max]".to_string()),
                    params: vec![
                        ParamSchema {
                            name: "min".to_string(),
                            ty: TypeRef {
                                name: "Int".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                        ParamSchema {
                            name: "max".to_string(),
                            ty: TypeRef {
                                name: "Int".to_string(),
                                nullable: false,
                                args: Vec::new(),
                            },
                            is_mut: false,
                            is_optional: false,
                            docs: None,
                        },
                    ],
                    returns: Some(TypeRef {
                        name: "Int".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.random".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "step".to_string(),
                    docs: Some(
                        "Advances the simulation by dt and returns the state digest".to_string(),
                    ),
                    params: vec![ParamSchema {
                        name: "dt".to_string(),
                        ty: TypeRef {
                            name: "Float".to_string(),
                            nullable: false,
                            args: Vec::new(),
                        },
                        is_mut: false,
                        is_optional: false,
                        docs: None,
                    }],
                    returns: Some(TypeRef {
                        name: "Int".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
                FunctionSchema {
                    name: "digest".to_string(),
                    docs: Some(
                        "Computes the deterministic state digest of the current world".to_string(),
                    ),
                    params: Vec::new(),
                    returns: Some(TypeRef {
                        name: "Int".to_string(),
                        nullable: false,
                        args: Vec::new(),
                    }),
                    is_async: false,
                    mutates_receiver: false,
                    capabilities: vec!["poppy.ecs".to_string()],
                    subject: None,
                    deprecated: None,
                },
            ],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poppy_schema_validates_cleanly() {
        let schema = poppy_schema();
        let problems = schema.validate();
        assert!(problems.is_ok(), "schema problems: {problems:?}");
    }

    #[test]
    fn test_poppy_schema_declared_capabilities() {
        let schema = poppy_schema();
        let caps = schema.declared_capabilities();
        assert!(caps.allows(&aipo_host::Capability::parse("poppy.ecs").unwrap()));
        assert!(caps.allows(&aipo_host::Capability::parse("poppy.random").unwrap()));
    }
}

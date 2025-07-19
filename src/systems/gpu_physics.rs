use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
};
use bytemuck::{Pod, Zeroable};
use crate::components::*;
use crate::resources::*;

/// GPU-compatible orbital state data structure
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuOrbitalState {
    /// Position vector in kilometers (x, y, z, mass)
    pub position: [f32; 4],
    /// Velocity vector in km/s (x, y, z, padding)
    pub velocity: [f32; 4],
}

impl GpuOrbitalState {
    pub fn from_orbital_state(state: &OrbitalState) -> Self {
        Self {
            position: [state.position.x, state.position.y, state.position.z, state.mass as f32],
            velocity: [state.velocity.x, state.velocity.y, state.velocity.z, 0.0],
        }
    }

    pub fn to_orbital_state(&self) -> OrbitalState {
        OrbitalState::new(
            Vec3::new(self.position[0], self.position[1], self.position[2]),
            Vec3::new(self.velocity[0], self.velocity[1], self.velocity[2]),
            self.position[3] as f64,
        )
    }
}

/// Resource to manage GPU physics buffers
#[derive(Resource)]
pub struct GpuPhysicsBuffers {
    pub orbital_states_buffer: Buffer,
    pub physics_params_buffer: Buffer,
    pub object_count: usize,
}

/// Physics parameters for GPU compute shader
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuPhysicsParams {
    /// Gravitational parameter (GM) in m³/s²
    pub gm: f32,
    /// Time step in seconds
    pub dt: f32,
    /// Number of objects to process
    pub object_count: u32,
    /// Padding for alignment
    pub _padding: u32,
}

/// Resource for GPU physics compute pipeline
#[derive(Resource)]
pub struct GpuPhysicsPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub compute_pipeline: ComputePipeline,
    pub bind_group: Option<BindGroup>,
}

/// Plugin to set up GPU physics
pub struct GpuPhysicsPlugin;

impl Plugin for GpuPhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Add to render app for GPU operations
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(Render, prepare_gpu_physics_buffers.in_set(RenderSet::Prepare))
            .add_systems(Render, run_gpu_physics.in_set(RenderSet::Queue));
    }

    fn finish(&self, app: &mut App) {
        // Initialize GPU resources
        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world.resource::<RenderDevice>();
        
        // Create compute pipeline
        let pipeline = create_gpu_physics_pipeline(render_device);
        render_app.world.insert_resource(pipeline);
    }
}

/// Create the GPU physics compute pipeline
fn create_gpu_physics_pipeline(render_device: &RenderDevice) -> GpuPhysicsPipeline {
    // Load compute shader from assets
    let shader_source = include_str!("../../assets/shaders/orbital_physics.wgsl");
    let shader_handle = render_device.create_shader_module(ShaderModuleDescriptor {
        label: Some("orbital_physics_compute"),
        source: ShaderSource::Wgsl(shader_source.into()),
    });

    // Create bind group layout
    let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("gpu_physics_bind_group_layout"),
        entries: &[
            // Orbital states buffer (read/write)
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Physics parameters buffer (read-only)
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    // Create compute pipeline
    let compute_pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("gpu_physics_pipeline"),
        layout: Some(&render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("gpu_physics_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        })),
        module: &shader_handle,
        entry_point: "main",
    });

    GpuPhysicsPipeline {
        bind_group_layout,
        compute_pipeline,
        bind_group: None,
    }
}

/// System to prepare GPU buffers with current orbital data
fn prepare_gpu_physics_buffers(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    orbital_query: Query<&OrbitalState, With<PhysicsObject>>,
    constants: Res<Constants>,
    sim_time: Res<SimulationTime>,
) {
    // Collect orbital states
    let orbital_states: Vec<GpuOrbitalState> = orbital_query
        .iter()
        .map(GpuOrbitalState::from_orbital_state)
        .collect();

    if orbital_states.is_empty() {
        return;
    }

    // Create physics parameters
    let physics_params = GpuPhysicsParams {
        gm: constants.gravitational_parameter as f32,
        dt: sim_time.timestep as f32,
        object_count: orbital_states.len() as u32,
        _padding: 0,
    };

    // Create or update orbital states buffer
    let orbital_states_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("orbital_states_buffer"),
        contents: bytemuck::cast_slice(&orbital_states),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
    });

    // Create physics parameters buffer
    let physics_params_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("physics_params_buffer"),
        contents: bytemuck::cast_slice(&[physics_params]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    // Store buffers as resource
    commands.insert_resource(GpuPhysicsBuffers {
        orbital_states_buffer,
        physics_params_buffer,
        object_count: orbital_states.len(),
    });
}

/// System to run GPU physics computation
fn run_gpu_physics(
    mut pipeline: ResMut<GpuPhysicsPipeline>,
    buffers: Res<GpuPhysicsBuffers>,
    render_device: Res<RenderDevice>,
    mut render_context: ResMut<RenderContext>,
) {
    // Create bind group if needed
    if pipeline.bind_group.is_none() {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("gpu_physics_bind_group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffers.orbital_states_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.physics_params_buffer.as_entire_binding(),
                },
            ],
        });
        pipeline.bind_group = Some(bind_group);
    }

    // Dispatch compute shader
    if let Some(ref bind_group) = pipeline.bind_group {
        let mut compute_pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("gpu_physics_pass"),
            });

        compute_pass.set_pipeline(&pipeline.compute_pipeline);
        compute_pass.set_bind_group(0, bind_group, &[]);
        
        // Dispatch with workgroups of 64 threads each
        let workgroup_count = (buffers.object_count + 63) / 64;
        compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);
    }
}

/// System to copy GPU results back to CPU (runs on main thread)
pub fn gpu_physics_readback_system(
    mut orbital_query: Query<&mut OrbitalState, With<PhysicsObject>>,
    // This system would need to read back GPU buffer results
    // Implementation would depend on Bevy's async GPU readback capabilities
) {
    // TODO: Implement GPU->CPU readback
    // This is complex in Bevy and might require async operations
    // For now, we'll use a hybrid approach where GPU computes and CPU reads back
}
use std::collections::HashMap;
use std::sync::Arc;

use crate::graphics::{BlendMode, PipelineData};
use vulkano::pipeline::graphics::color_blend::ColorComponents;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::StateMode;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::layout::DescriptorSetLayout,
    pipeline::{
        graphics::color_blend::{
            AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
            LogicOp,
        },
        graphics::vertex_input::VertexDefinition,
        GraphicsPipeline,
    },
    device::Device,
    render_pass::{Subpass, RenderPass},
    shader::EntryPoint,
};

#[derive(Clone, Copy)]
pub enum VertexTopology {
    PointList,
    TriangleFan,
    TriangleList,
    TriangleStrip,
}

pub enum ShaderType {
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEval,
    Geometry,
    Default,
}

pub type ShaderId = usize;

pub struct Shader<'s> {
    pub vertex: EntryPoint<'s>,
    pub fragment: EntryPoint<'s>,
    pub topology: VertexTopology,
    // pub vertex_definition: Vd,
}

pub struct ShaderProgram {
    pipelines: PipelineObjectSet,
    current_mode: BlendMode,
}

pub trait ShaderHandle {
    fn draw(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipe_data: Box<PipelineData>,
    );
    // fn set_blend_mode(&mut self, mode: BlendMode);
    fn blend_mode(&self) -> BlendMode;
    fn layout(&self) -> &[Arc<DescriptorSetLayout>];
    fn pipeline(&self) -> Arc<GraphicsPipeline>;
}

impl ShaderHandle for ShaderProgram {
    fn draw(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipe_data: Box<PipelineData>,
    ) {
        command_buffer.bind_pipeline_graphics(self.pipeline().clone());

        let layout = self.layout()[1].clone();

        let (buffers, descriptors, v_count, i_count) = pipe_data.flush();

        let set =
            vulkano::descriptor_set::PersistentDescriptorSet::new(layout.clone(), descriptors)
                .unwrap();

        command_buffer.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            self.pipeline().layout().clone(),
            1,
            set,
        );

        command_buffer.bind_vertex_buffers(0, buffers);

        command_buffer.draw(v_count, i_count, 0, 0).unwrap();
    }

    fn blend_mode(&self) -> BlendMode {
        self.current_mode
    }

    fn layout(&self) -> &[Arc<DescriptorSetLayout>] {
        self.pipelines
            .get(&self.current_mode)
            .unwrap()
            .layout()
            .set_layouts()
    }

    fn pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipelines.get(&self.current_mode).unwrap().clone()
    }
}

impl ShaderProgram {
    pub fn new<Vd>(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        vertex_type: Vd,
        vertex_order: VertexTopology,
        vertex_shader: EntryPoint,
        fragment_shader: EntryPoint,
        blend: BlendMode,
    ) -> Self
    // ) -> Result<Self, GraphicsPipelineCreationError>
    where
        Vd: VertexDefinition + 'static + Sync + Send,
    {
        let po = new_pipeline(
            device,
            render_pass,
            vertex_type,
            vertex_order,
            vertex_shader,
            fragment_shader,
            blend,
        );

        let mut pos = PipelineObjectSet::new(16);
        pos.insert(blend, po);

        Self {
            pipelines: pos,
            current_mode: blend,
        }
    }

    pub fn from_pipeline(mode: BlendMode, pipeline: Arc<GraphicsPipeline>) -> Self {
        let mut pipeline_os = PipelineObjectSet::new(16);
        pipeline_os.insert(mode, pipeline);
        Self {
            pipelines: pipeline_os,
            current_mode: mode,
        }
    }
}

// This structure is to store multiple pipelines for different blend modes.
pub struct PipelineObjectSet {
    pipelines: HashMap<BlendMode, Arc<GraphicsPipeline>>,
}

impl PipelineObjectSet {
    pub fn new(cap: usize) -> Self {
        Self {
            pipelines: HashMap::with_capacity(cap),
        }
    }

    pub fn insert(&mut self, blend_mode: BlendMode, pipeline: Arc<GraphicsPipeline>) {
        self.pipelines.insert(blend_mode, pipeline);
    }

    pub fn get(&self, blend_mode: &BlendMode) -> Option<&Arc<GraphicsPipeline>> {
        self.pipelines.get(blend_mode)
    }

    // pub fn mode(&self, mode: &BlendMode) -> Result<&GraphicsPipeline, GraphicsError> {
    //     match self.pipelines.get(&mode) {
    //         Some(po) => Ok(po),
    //         None => {},
    //     }
    // }
}

pub fn new_pipeline<Vd>(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    vertex_type: Vd,
    vertex_order: VertexTopology,
    vertex_shader: EntryPoint,
    fragment_shader: EntryPoint,
    blend: BlendMode,
// ) -> Result<Arc<GraphicsPipeline>, GraphicsPipelineCreationError>
) -> Arc<GraphicsPipeline>
where
    Vd: VertexDefinition + 'static + Sync + Send,
{
    let mut pipeline = GraphicsPipeline::start()
        .vertex_input_state::<Vd>(vertex_type)
        .vertex_shader(vertex_shader, ())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader, ())
        .color_blend_state(blend.into())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap());

    pipeline = match vertex_order {
        VertexTopology::PointList => pipeline
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList)),
        VertexTopology::TriangleFan => pipeline.input_assembly_state(
            InputAssemblyState::new().topology(PrimitiveTopology::TriangleFan),
        ),
        VertexTopology::TriangleList => pipeline.input_assembly_state(
            InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
        ),
        VertexTopology::TriangleStrip => pipeline.input_assembly_state(
            InputAssemblyState::new().topology(PrimitiveTopology::TriangleStrip),
        ),
    };

    pipeline.build(device.clone()).unwrap()
}

impl From<BlendMode> for ColorBlendState {
    fn from(blend_mode: BlendMode) -> Self {
        let mut logic_op: Option<StateMode<LogicOp>> = None;
        let mut attach: Option<AttachmentBlend> = None;
        let blend_constants: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // TODO implement these.

        match blend_mode {
            BlendMode::Add => {
                attach = Some(AttachmentBlend {
                    color_op: BlendOp::Add,
                    color_source: BlendFactor::One,
                    color_destination: BlendFactor::One,
                    alpha_op: BlendOp::Add,
                    alpha_source: BlendFactor::One,
                    alpha_destination: BlendFactor::One,
                });
            }
            BlendMode::Subtract => {
                attach = Some(AttachmentBlend {
                    color_op: BlendOp::Subtract,
                    color_source: BlendFactor::One,
                    color_destination: BlendFactor::One,
                    alpha_op: BlendOp::Subtract,
                    alpha_source: BlendFactor::One,
                    alpha_destination: BlendFactor::One,
                });
            }
            BlendMode::Alpha => {
                attach = Some(AttachmentBlend::alpha());
            }
            BlendMode::Invert => {
                logic_op = Some(StateMode::Fixed(LogicOp::Invert));
            }
        };

        return ColorBlendState {
            logic_op: logic_op,
            attachments: vec![ColorBlendAttachmentState {
                blend: attach,
                color_write_mask: ColorComponents::all(),
                color_write_enable: StateMode::Fixed(true),
            }],
            blend_constants: StateMode::Fixed(blend_constants),
        };
    }
}

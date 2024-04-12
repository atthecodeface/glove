//a Imports
use std::borrow::Cow;

use crate::{Accelerate, KernelArgs};
use ic_base::utils::rtc::run_to_completion as rtc;

//a Support types
//tp Pipeline
#[derive(Debug, Clone, Copy)]
pub struct Pipeline(usize);

//ip Pipeline
impl Pipeline {
    fn as_usize(&self) -> usize {
        self.0
    }
}

//tp From<usize> for Pipeline
impl From<usize> for Pipeline {
    fn from(n: usize) -> Pipeline {
        Pipeline(n)
    }
}

//tp Shader
#[derive(Debug, Clone, Copy)]
pub struct Shader(usize);

//ip Shader
impl Shader {
    fn as_usize(&self) -> usize {
        self.0
    }
}

//ip From<usize> for Shader
impl From<usize> for Shader {
    fn from(n: usize) -> Shader {
        Shader(n)
    }
}

//tp BufferType
pub enum BufferType {
    HostSrc,
    GpuData,
    GpuUniform,
    HostDst,
}

//a AccelWgpu
//tp AccelWgpu
#[derive(Debug)]
pub struct AccelWgpu {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shaders: Vec<wgpu::ShaderModule>,
    pipelines: Vec<(wgpu::ComputePipeline, Vec<wgpu::BindGroupLayout>)>,
}

//ip AccelWgpu
impl AccelWgpu {
    //cp new
    pub fn new() -> Self {
        // Instantiates instance of WebGPU
        let instance = wgpu::Instance::default();

        // `request_adapter` instantiates the general connection to the GPU
        let adapter =
            rtc(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();

        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        let (device, queue) = rtc(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))
        .unwrap();
        let shaders = vec![];
        let pipelines = vec![];
        Self {
            instance,
            adapter,
            device,
            queue,
            shaders,
            pipelines,
        }
    }

    //mp add_shader
    pub fn add_shader(
        &mut self,
        code: &str,
        opt_label: Option<&'static str>,
    ) -> Result<Shader, String> {
        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: opt_label,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
        };

        // Loads the shader from WGSL
        let sm = self.device.create_shader_module(shader_desc);
        let n = self.shaders.len();
        self.shaders.push(sm);
        Ok(n.into())
    }

    //ap device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    //ap queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    //ap shader
    pub fn shader(&self, shader: Shader) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(shader.as_usize())
    }

    //ap shader_err
    pub fn shader_err(&self, shader: Shader) -> Result<&wgpu::ShaderModule, String> {
        self.shader(shader).ok_or("Bad shader index".into())
    }

    //ap pipeline
    pub fn pipeline(&self, pipeline: Pipeline) -> Option<&wgpu::ComputePipeline> {
        self.pipelines.get(pipeline.as_usize()).map(|(p, _bg)| p)
    }

    //ap pipeline_err
    pub fn pipeline_err(&self, pipeline: Pipeline) -> Result<&wgpu::ComputePipeline, String> {
        self.pipeline(pipeline).ok_or("Bad pipeline index".into())
    }

    //mp create_pipeline_layout
    /// A pipeline layout can be used by many pipelines
    pub fn create_pipeline_layout(
        &self,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        let l = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        };
        self.device.create_pipeline_layout(&l)
    }

    //mp create_pipeline
    pub fn create_pipeline(
        &mut self,
        shader: Shader,
        entry_point: &str,
        layout: Option<&wgpu::PipelineLayout>,
        num_bind_groups: usize,
    ) -> Result<Pipeline, String> {
        let pipeline_desc = wgpu::ComputePipelineDescriptor {
            label: None,
            layout,
            module: self.shader_err(shader)?,
            entry_point,
        };
        let pipeline = self.device().create_compute_pipeline(&pipeline_desc);
        let n = self.pipelines.len();
        let mut bgl = vec![];
        for i in 0..num_bind_groups {
            bgl.push(pipeline.get_bind_group_layout(i as u32));
        }
        self.pipelines.push((pipeline, bgl));
        Ok(n.into())
    }

    //mp create_buffer
    pub fn create_buffer(
        &self,
        buffer_type: BufferType,
        buffer_size: usize,
        opt_label: Option<&'static str>,
    ) -> wgpu::Buffer {
        let usage = match buffer_type {
            BufferType::HostDst => wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            BufferType::HostSrc => wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            BufferType::GpuData => {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
            }
            BufferType::GpuUniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };
        let bd = wgpu::BufferDescriptor {
            label: opt_label,
            size: buffer_size as u64,
            usage,
            mapped_at_creation: false,
        };
        self.device.create_buffer(&bd)
    }

    //mp block
    pub fn block(&self) {
        self.device()
            .poll(wgpu::Maintain::wait())
            .panic_on_timeout();
    }

    //zz All done
}

//a ImageAccelerator
//tp ImageAccelerator
#[derive(Debug)]
pub struct ImageAccelerator {
    accelerator: AccelWgpu,
    buffer_size: usize,
    compute_vec_sqrt: Pipeline,
    compute_vec_square: Pipeline,
    compute_window_sum_x: Pipeline,
    compute_window_sum_y: Pipeline,
    compute_window_mean: Pipeline,
    compute_window_var: Pipeline,
    compute_window_var_scaled: Pipeline,
    /// Buffer for data that gets copied TO the GPU shader_in_out_data_buffer
    input_buffer: wgpu::Buffer,
    /// Buffer for data that gets copied FROM the GPU shader_in_out_data_buffer
    output_buffer: wgpu::Buffer,
    /// Buffer for data that gets copied TO the GPU shader_uniform_buffer
    uniform_buffer: wgpu::Buffer,
    /// The buffer in the GPU for *pure* input data - this is 'rs2' as it were; img_a if required
    shader_in_data_buffer: wgpu::Buffer,
    /// The buffer in the GPU for *pure* input data - this is 'rs2' as it were; img_b if required
    shader_in_data_b_buffer: wgpu::Buffer,
    /// The buffer in the GPU for in-out data
    shader_in_out_data_buffer: wgpu::Buffer,
    /// The buffer in the GPU for in-out data
    shader_uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

//ip ImageAccelerator
impl ImageAccelerator {
    pub fn new(mut accelerator: AccelWgpu, buffer_size: usize) -> Result<Self, String> {
        let file_text = std::fs::read_to_string("shader.wgsl")
            .map_err(|e| format!("Error reading json shader file {}", e))?;
        // let cs_module = accelerator.add_shader(include_str!("shader.wgsl"), None)?;
        let cs_module = accelerator.add_shader(&file_text, None)?;
        let in_out_data_bgle = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let in_data_bgle = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let in_data_b_bgle = wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let uniform_bgle = wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bgl = accelerator
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[in_out_data_bgle, in_data_bgle, in_data_b_bgle, uniform_bgle],
            });
        let pl = accelerator.create_pipeline_layout(&[&bgl]);
        let compute_vec_sqrt =
            accelerator.create_pipeline(cs_module, "compute_vec_sqrt", Some(&pl), 1)?;
        let compute_vec_square =
            accelerator.create_pipeline(cs_module, "compute_vec_square", Some(&pl), 1)?;
        let compute_window_sum_x =
            accelerator.create_pipeline(cs_module, "compute_window_sum_x", Some(&pl), 1)?;
        let compute_window_sum_y =
            accelerator.create_pipeline(cs_module, "compute_window_sum_y", Some(&pl), 1)?;
        let compute_window_mean =
            accelerator.create_pipeline(cs_module, "compute_window_mean", Some(&pl), 1)?;
        let compute_window_var =
            accelerator.create_pipeline(cs_module, "compute_window_var", Some(&pl), 1)?;
        let compute_window_var_scaled =
            accelerator.create_pipeline(cs_module, "compute_window_var_scaled", Some(&pl), 1)?;
        let input_buffer =
            accelerator.create_buffer(BufferType::HostSrc, buffer_size, Some("Input"));
        let shader_in_data_buffer =
            accelerator.create_buffer(BufferType::GpuData, buffer_size, Some("Shader In Data"));
        let shader_in_data_b_buffer =
            accelerator.create_buffer(BufferType::GpuData, buffer_size, Some("Shader In B Data"));
        let shader_in_out_data_buffer =
            accelerator.create_buffer(BufferType::GpuData, buffer_size, Some("Shader InOut Data"));
        let shader_uniform_buffer = accelerator.create_buffer(
            BufferType::GpuUniform,
            std::mem::size_of::<KernelArgs>(),
            Some("Shader Uniform"),
        );
        let uniform_buffer = accelerator.create_buffer(
            BufferType::HostSrc,
            std::mem::size_of::<KernelArgs>(),
            Some("Uniform"),
        );
        let output_buffer =
            accelerator.create_buffer(BufferType::HostDst, buffer_size, Some("Output"));
        let bind_group = accelerator
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: shader_in_out_data_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: shader_in_data_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: shader_in_data_b_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: shader_uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        Ok(Self {
            accelerator,
            buffer_size,
            compute_vec_sqrt,
            compute_vec_square,
            compute_window_sum_x,
            compute_window_sum_y,
            compute_window_mean,
            compute_window_var,
            compute_window_var_scaled,
            input_buffer,
            uniform_buffer,
            output_buffer,
            shader_in_out_data_buffer,
            shader_in_data_buffer,
            shader_in_data_b_buffer,
            shader_uniform_buffer,
            bind_group,
        })
    }

    //mp cmd_buffer
    fn cmd_buffer(
        &self,
        number_of_ops: usize,
        pipeline: Pipeline,
    ) -> Result<wgpu::CommandBuffer, String> {
        let mut encoder = self
            .accelerator
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            &self.input_buffer,
            0,
            &self.shader_in_data_buffer,
            0,
            self.buffer_size as u64,
        );
        encoder.copy_buffer_to_buffer(
            &self.uniform_buffer,
            0,
            &self.shader_uniform_buffer,
            0,
            std::mem::size_of::<KernelArgs>() as u64,
        );
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_pipeline(self.accelerator.pipeline_err(pipeline)?);
            cpass.insert_debug_marker("image accelerator slice");
            // Number of cells to run, the (x,y,z) size of item being processed
            // If number of ops is bigger than 65536 then split - but then we need to specify an offset in the uniform?
            // Should set_push_constants
            cpass.dispatch_workgroups(number_of_ops as u32, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &self.shader_in_out_data_buffer,
            0,
            &self.output_buffer,
            0,
            self.buffer_size as u64,
        );

        Ok(encoder.finish())
    }

    //mp create_cmd_buffer
    fn create_cmd_buffer(
        &self,
        number_of_ops: [usize; 3],
        pipeline: Pipeline,
        buffers_in: &[((&wgpu::Buffer, usize), (&wgpu::Buffer, usize), usize)],
        buffers_out: &[((&wgpu::Buffer, usize), (&wgpu::Buffer, usize), usize)],
    ) -> Result<wgpu::CommandBuffer, String> {
        let mut encoder = self
            .accelerator
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        for (bi, bo, size) in buffers_in {
            encoder.copy_buffer_to_buffer(bi.0, bi.1 as u64, bo.0, bo.1 as u64, *size as u64);
        }
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.set_pipeline(self.accelerator.pipeline_err(pipeline)?);
        cpass.dispatch_workgroups(
            number_of_ops[0] as u32,
            number_of_ops[1] as u32,
            number_of_ops[2] as u32,
        );
        drop(cpass); // so encoder can be mutable again
        for (bi, bo, size) in buffers_out {
            encoder.copy_buffer_to_buffer(bi.0, bi.1 as u64, bo.0, bo.1 as u64, *size as u64);
        }

        Ok(encoder.finish())
    }

    //mp copy_to_input
    fn copy_to_input(&self, src_data: &[u8]) -> Result<(), String> {
        let byte_size = src_data.len() as u64;
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.input_buffer.slice(0..byte_size);
        buffer_slice.map_async(wgpu::MapMode::Write, move |v| sender.send(v).unwrap());
        self.accelerator.block();
        let Ok(Ok(())) = receiver.recv() else {
            return Err("Failed to run compute on gpu!".into());
        };
        buffer_slice
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(src_data));
        self.input_buffer.unmap();
        Ok(())
    }

    //mp copy_uniform
    fn copy_uniform(&self, args: &KernelArgs) -> Result<(), String> {
        let byte_size = std::mem::size_of::<KernelArgs>() as u64;
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.uniform_buffer.slice(0..byte_size);
        buffer_slice.map_async(wgpu::MapMode::Write, move |v| sender.send(v).unwrap());
        self.accelerator.block();
        let Ok(Ok(())) = receiver.recv() else {
            return Err("Failed to run compute on gpu!".into());
        };
        buffer_slice
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(&[*args]));
        self.uniform_buffer.unmap();
        Ok(())
    }

    //mp copy_output
    fn copy_output(&self, byte_size: u64) -> Result<wgpu::BufferView, String> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.output_buffer.slice(0..byte_size);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        self.accelerator.block();
        let Ok(Ok(())) = receiver.recv() else {
            return Err("Failed to run compute on gpu!".into());
        };
        Ok(buffer_slice.get_mapped_range())
    }

    //mp run
    fn run<F: Fn(&KernelArgs, &[f32], &mut [f32]) -> Result<(), String>>(
        &self,
        args: &KernelArgs,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
        cmd_buffer: wgpu::CommandBuffer,
        callback: &F,
    ) -> Result<(), String> {
        let byte_size = std::mem::size_of_val(out_data) as u64;
        if let Some(src_data) = src_data {
            self.copy_to_input(bytemuck::cast_slice(src_data))?;
        } else {
            self.copy_to_input(bytemuck::cast_slice(out_data))?;
        }
        self.copy_uniform(args)?;
        self.accelerator.queue().submit(Some(cmd_buffer));
        let data = self.copy_output(byte_size)?;
        callback(args, bytemuck::cast_slice(&data), out_data)?;
        // Must drop data so that self.output_buffer has no BufferView's
        drop(data);
        // Must unmap self.output_buffer so it can be used in the future
        self.output_buffer.unmap();
        Ok(())
    }

    //zz All done
}

//ip Accelerate for ImageAccelerator
impl Accelerate for ImageAccelerator {
    //mp run_shader
    fn run_shader(
        &self,
        shader: &str,
        args: &KernelArgs,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
    ) -> Result<bool, String> {
        match shader {
            "sqrt" => {
                let cmd_buffer =
                    self.cmd_buffer(args.width() * args.height() / 64, self.compute_vec_sqrt)?;
                self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                    o.copy_from_slice(s);
                    Ok(())
                })?;
                Ok(true)
            }
            "square" => {
                let cmd_buffer =
                    self.cmd_buffer(args.width() * args.height() / 64, self.compute_vec_square)?;
                self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                    o.copy_from_slice(s);
                    Ok(())
                })?;
                Ok(true)
            }
            "window_sum_x" => {
                let cmd_buffer =
                    self.cmd_buffer(args.width() * args.height() / 64, self.compute_window_sum_x)?;
                self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                    o.copy_from_slice(s);
                    Ok(())
                })?;
                Ok(true)
            }
            "window_sum_y" => {
                let cmd_buffer = self.create_cmd_buffer(
                    [args.width() / 64, args.height(), 1],
                    self.compute_window_sum_y,
                    &[
                        (
                            (&self.input_buffer, 0),
                            (&self.shader_in_data_buffer, 0),
                            self.buffer_size,
                        ),
                        (
                            (&self.uniform_buffer, 0),
                            (&self.shader_uniform_buffer, 0),
                            std::mem::size_of::<KernelArgs>(),
                        ),
                    ],
                    &[(
                        (&self.shader_in_out_data_buffer, 0),
                        (&self.output_buffer, 0),
                        self.buffer_size,
                    )],
                )?;
                self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                    o.copy_from_slice(s);
                    Ok(())
                })?;
                Ok(true)
            }
            "window_mean" => {
                let cmd_buffer = self.create_cmd_buffer(
                    [args.width() * args.height() / 256, 1, 1],
                    self.compute_window_mean,
                    &[
                        (
                            (&self.input_buffer, 0),
                            (&self.shader_in_data_buffer, 0),
                            self.buffer_size,
                        ),
                        (
                            (&self.uniform_buffer, 0),
                            (&self.shader_uniform_buffer, 0),
                            std::mem::size_of::<KernelArgs>(),
                        ),
                    ],
                    &[(
                        (&self.shader_in_out_data_buffer, 0),
                        (&self.output_buffer, 0),
                        self.buffer_size,
                    )],
                )?;
                self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                    o.copy_from_slice(s);
                    Ok(())
                })?;
                Ok(true)
            }
            "window_var" => {
                let cmd_buffer = self.create_cmd_buffer(
                    [args.width() * args.height() / 256, 1, 1],
                    self.compute_window_var,
                    &[
                        (
                            (&self.input_buffer, 0),
                            (&self.shader_in_data_buffer, 0),
                            self.buffer_size,
                        ),
                        (
                            (&self.uniform_buffer, 0),
                            (&self.shader_uniform_buffer, 0),
                            std::mem::size_of::<KernelArgs>(),
                        ),
                    ],
                    &[(
                        (&self.shader_in_out_data_buffer, 0),
                        (&self.output_buffer, 0),
                        self.buffer_size,
                    )],
                )?;
                self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                    o.copy_from_slice(s);
                    Ok(())
                })?;
                Ok(true)
            }
            "window_var_scaled" => {
                let cmd_buffer = self.create_cmd_buffer(
                    [args.width() * args.height() / 256, 1, 1],
                    self.compute_window_var_scaled,
                    &[
                        (
                            (&self.input_buffer, 0),
                            (&self.shader_in_data_buffer, 0),
                            self.buffer_size,
                        ),
                        (
                            (&self.uniform_buffer, 0),
                            (&self.shader_uniform_buffer, 0),
                            std::mem::size_of::<KernelArgs>(),
                        ),
                    ],
                    &[(
                        (&self.shader_in_out_data_buffer, 0),
                        (&self.output_buffer, 0),
                        self.buffer_size,
                    )],
                )?;
                self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                    o.copy_from_slice(s);
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
